use mictylish::error::{EvalError, ResolveError};
use mictylish::eval::{eval_program, EvalEnv};
use mictylish::parser::parse_program;
use mictylish::resolver::Resolver;
use mictylish::value::Value;

fn run(source: &str) -> EvalEnv {
    let program = parse_program(source).expect("parse");
    let mut resolver = Resolver::new();
    resolver.resolve_program(&program).expect("resolve");
    let mut env = EvalEnv::new();
    eval_program(&mut env, &program).expect("eval");
    env
}

#[test]
fn set_updates_mutable_binding() {
    let env = run("let mut x = 1 set x = 2");
    assert_eq!(env.get("x"), Some(&Value::Int(2)));
}

#[test]
fn set_can_change_type() {
    let env = run(r#"let mut x = 1 set x = "hello""#);
    assert_eq!(env.get("x"), Some(&Value::String("hello".to_string())));
}

#[test]
fn set_multiple_times() {
    let env = run("let mut x = 0 set x = 1 set x = 2 set x = 3");
    assert_eq!(env.get("x"), Some(&Value::Int(3)));
}

#[test]
fn set_uses_current_env_in_rhs() {
    let env = run("let a = 10 let mut x = 0 set x = a");
    assert_eq!(env.get("x"), Some(&Value::Int(10)));
}

#[test]
fn set_on_immutable_binding_is_rejected() {
    let program = parse_program("let x = 1 set x = 2").unwrap();
    let mut resolver = Resolver::new();
    let err = resolver.resolve_program(&program).expect_err("not mutable");
    assert!(matches!(err, ResolveError::SetNotMutable(_)));
}

#[test]
fn set_on_undefined_name_is_rejected() {
    let program = parse_program("set x = 1").unwrap();
    let mut resolver = Resolver::new();
    let err = resolver.resolve_program(&program).expect_err("not defined");
    assert!(matches!(err, ResolveError::SetUndefined(_)));
}

#[test]
fn set_parse_correct_syntax() {
    let program = parse_program("let mut x = 1 set x = 2").expect("parse");
    assert_eq!(program.stmts.len(), 2);
}

#[test]
fn set_parse_missing_equal() {
    let err = parse_program("let mut x = 1 set x 2").expect_err("missing =");
    assert!(err.message.contains("'=' after set binding"));
}

#[test]
fn set_inside_repl_session() {
    let mut resolver = Resolver::new();
    let mut env = EvalEnv::new();

    let p1 = parse_program("let mut counter = 0").unwrap();
    resolver.resolve_program(&p1).unwrap();
    eval_program(&mut env, &p1).unwrap();

    let p2 = parse_program("set counter = 1").unwrap();
    resolver.resolve_program(&p2).unwrap();
    eval_program(&mut env, &p2).unwrap();

    assert_eq!(env.get("counter"), Some(&Value::Int(1)));
}

#[test]
fn set_rejects_immutable_across_repl_lines() {
    let mut resolver = Resolver::new();
    let p1 = parse_program("let x = 1").unwrap();
    resolver.resolve_program(&p1).unwrap();

    let p2 = parse_program("set x = 2").unwrap();
    let err = resolver.resolve_program(&p2).expect_err("not mutable");
    assert!(matches!(err, ResolveError::SetNotMutable(_)));
}
