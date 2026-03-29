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
fn match_int_literal() {
    let env = run("let x = match 1 do 1 -> 10 2 -> 20 end");
    assert_eq!(env.get("x"), Some(&Value::Int(10)));
}

#[test]
fn match_second_arm() {
    let env = run("let x = match 2 do 1 -> 10 2 -> 20 end");
    assert_eq!(env.get("x"), Some(&Value::Int(20)));
}

#[test]
fn match_string_literal() {
    let env = run("let x = match \"hi\" do \"bye\" -> 0 \"hi\" -> 1 end");
    assert_eq!(env.get("x"), Some(&Value::Int(1)));
}

#[test]
fn match_wildcard() {
    let env = run("let x = match 99 do 1 -> 10 _ -> 0 end");
    assert_eq!(env.get("x"), Some(&Value::Int(0)));
}

#[test]
fn match_var_binding() {
    let env = run("let x = match 5 do n -> n end");
    assert_eq!(env.get("x"), Some(&Value::Int(5)));
}

#[test]
fn match_list_pattern() {
    let env = run("let x = match [1, 2] do [a, b] -> [b, a] end");
    assert_eq!(
        env.get("x"),
        Some(&Value::List(vec![Value::Int(2), Value::Int(1)]))
    );
}

#[test]
fn match_list_length_mismatch_falls_through() {
    let env = run("let x = match [1] do [a, b] -> 0 _ -> 1 end");
    assert_eq!(env.get("x"), Some(&Value::Int(1)));
}

#[test]
fn match_exhausted_is_error() {
    let program = parse_program("let x = match 3 do 1 -> 10 2 -> 20 end").unwrap();
    let mut resolver = Resolver::new();
    resolver.resolve_program(&program).unwrap();
    let mut env = EvalEnv::new();
    let err = eval_program(&mut env, &program).expect_err("no arm matched");
    assert!(matches!(err, EvalError::MatchExhausted(_)));
}

#[test]
fn match_in_pipe_chain() {
    let env = run(
        "let classify = fn x -> match x do 0 -> \"zero\" _ -> \"nonzero\" end end \
         let y = 0 |> classify",
    );
    assert_eq!(env.get("y"), Some(&Value::String("zero".to_string())));
}

#[test]
fn match_body_uses_outer_binding() {
    let env = run("let base = 100 let x = match 1 do _ -> base end");
    assert_eq!(env.get("x"), Some(&Value::Int(100)));
}

#[test]
fn match_parse_missing_do() {
    let err = parse_program("let x = match 1 1 -> 10 end").expect_err("missing do");
    assert!(err.message.contains("`do` after match subject"));
}

#[test]
fn match_parse_empty_arms() {
    let err = parse_program("let x = match 1 do end").expect_err("no arms");
    assert!(err.message.contains("at least one arm"));
}

#[test]
fn match_resolve_rejects_undefined_in_body() {
    let program = parse_program("let x = match 1 do _ -> z end").unwrap();
    let mut resolver = Resolver::new();
    let err = resolver.resolve_program(&program).expect_err("z undefined");
    assert!(matches!(err, ResolveError::Undefined(_)));
}

#[test]
fn match_resolve_var_pattern_binds_in_body() {
    let program = parse_program("let x = match 1 do val -> val end").unwrap();
    let mut resolver = Resolver::new();
    resolver.resolve_program(&program).expect("val is bound by pattern");
}
