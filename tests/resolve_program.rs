use mictylish::error::ResolveError;
use mictylish::parser::parse_program;
use mictylish::resolver::Resolver;

#[test]
fn sequential_lets_can_use_prior_binding() {
    let program = parse_program("let a = 1 let b = a").expect("parse");
    let mut resolver = Resolver::new();
    resolver.resolve_program(&program).expect("resolve");
    assert!(resolver.is_defined("a"));
    assert!(resolver.is_defined("b"));
}

#[test]
fn repl_session_accumulates_bindings() {
    let mut resolver = Resolver::new();
    resolver
        .resolve_program(&parse_program("let a = 1").unwrap())
        .unwrap();
    resolver
        .resolve_program(&parse_program("let b = a").unwrap())
        .unwrap();
    assert!(resolver.is_defined("b"));
}

#[test]
fn duplicate_let_in_same_program_is_rejected() {
    let program = parse_program("let x = 1 let x = 2").expect("parse");
    let mut resolver = Resolver::new();
    let err = resolver
        .resolve_program(&program)
        .expect_err("shadowing");
    assert!(matches!(err, ResolveError::Shadowing(_)));
}

#[test]
fn duplicate_let_across_repl_lines_is_rejected() {
    let mut resolver = Resolver::new();
    resolver
        .resolve_program(&parse_program("let x = 1").unwrap())
        .unwrap();
    let err = resolver
        .resolve_program(&parse_program("let x = 2").unwrap())
        .expect_err("shadowing");
    assert!(matches!(err, ResolveError::Shadowing(_)));
}

#[test]
fn undefined_variable_in_rhs_is_rejected() {
    let program = parse_program("let y = z").expect("parse");
    let mut resolver = Resolver::new();
    let err = resolver
        .resolve_program(&program)
        .expect_err("undefined");
    assert!(matches!(err, ResolveError::Undefined(_)));
}

#[test]
fn undefined_variable_in_pipe_is_rejected() {
    let program = parse_program("let y = 1 |> sink").expect("parse");
    let mut resolver = Resolver::new();
    let err = resolver
        .resolve_program(&program)
        .expect_err("undefined sink");
    assert!(matches!(err, ResolveError::Undefined(_)));
}

#[test]
fn undefined_variable_inside_list_is_rejected() {
    let program = parse_program("let y = [unknown]").expect("parse");
    let mut resolver = Resolver::new();
    let err = resolver
        .resolve_program(&program)
        .expect_err("undefined");
    assert!(matches!(err, ResolveError::Undefined(_)));
}

#[test]
fn pipe_prelude_identity_resolves_without_prior_let() {
    let program = parse_program("let y = 42 |> identity").expect("parse");
    let mut resolver = Resolver::new();
    resolver.resolve_program(&program).expect("identity is allowed as pipe target");
}

#[test]
fn pipe_rhs_must_be_identifier() {
    let program = parse_program("let y = 1 |> [2]").expect("parse");
    let mut resolver = Resolver::new();
    let err = resolver
        .resolve_program(&program)
        .expect_err("list on rhs");
    assert!(matches!(err, ResolveError::InvalidPipeRhs(_)));
}

#[test]
fn fn_body_can_use_its_parameter() {
    let program = parse_program("let id = fn x -> x end").expect("parse");
    let mut resolver = Resolver::new();
    resolver.resolve_program(&program).expect("x is function parameter");
}

#[test]
fn fn_body_rejects_undefined_name() {
    let program = parse_program("let f = fn x -> y end").expect("parse");
    let mut resolver = Resolver::new();
    let err = resolver
        .resolve_program(&program)
        .expect_err("y should be undefined");
    assert!(matches!(err, ResolveError::Undefined(_)));
}

#[test]
fn fn_parameter_shadowing_is_rejected() {
    let program = parse_program("let x = 1 let f = fn x -> x end").expect("parse");
    let mut resolver = Resolver::new();
    let err = resolver
        .resolve_program(&program)
        .expect_err("shadowing should be rejected");
    assert!(matches!(err, ResolveError::Shadowing(_)));
}

#[test]
fn pipe_to_user_defined_fn_resolves() {
    let program = parse_program("let id = fn x -> x end let y = 1 |> id").expect("parse");
    let mut resolver = Resolver::new();
    resolver.resolve_program(&program).expect("user function is valid pipe target");
}
