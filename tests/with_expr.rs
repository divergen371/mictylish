use mictylish::error::ResolveError;
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
fn with_single_binding_match() {
    let env = run("let x = with a <- 1 do a else 0 end");
    assert_eq!(env.get("x"), Some(&Value::Int(1)));
}

#[test]
fn with_single_binding_mismatch_goes_to_else() {
    let env = run("let x = with 99 <- 1 do 10 else 0 end");
    assert_eq!(env.get("x"), Some(&Value::Int(0)));
}

#[test]
fn with_multiple_bindings() {
    let env = run("let x = with a <- 1, b <- 2 do [a, b] else [] end");
    assert_eq!(
        env.get("x"),
        Some(&Value::List(vec![Value::Int(1), Value::Int(2)]))
    );
}

#[test]
fn with_second_binding_mismatch() {
    let env = run("let x = with a <- 1, 99 <- 2 do a else 0 end");
    assert_eq!(env.get("x"), Some(&Value::Int(0)));
}

#[test]
fn with_list_destructure() {
    let env = run("let pair = [3, 4] let x = with [a, b] <- pair do [b, a] else [] end");
    assert_eq!(
        env.get("x"),
        Some(&Value::List(vec![Value::Int(4), Value::Int(3)]))
    );
}

#[test]
fn with_list_destructure_mismatch() {
    let env = run("let triple = [1, 2, 3] let x = with [a, b] <- triple do a else 0 end");
    assert_eq!(env.get("x"), Some(&Value::Int(0)));
}

#[test]
fn with_body_uses_outer_scope() {
    let env = run("let base = 10 let x = with a <- 5 do [base, a] else [] end");
    assert_eq!(
        env.get("x"),
        Some(&Value::List(vec![Value::Int(10), Value::Int(5)]))
    );
}

#[test]
fn with_bindings_accumulate() {
    let env = run("let x = with a <- 1, b <- [a, 2] do b else [] end");
    assert_eq!(
        env.get("x"),
        Some(&Value::List(vec![Value::Int(1), Value::Int(2)]))
    );
}

#[test]
fn with_parse_missing_arrow() {
    let err = parse_program("let x = with a 1 do a else 0 end").expect_err("missing <-");
    assert!(err.message.contains("'<-' after with pattern"));
}

#[test]
fn with_parse_missing_do() {
    let err = parse_program("let x = with a <- 1 a else 0 end").expect_err("missing do");
    assert!(err.message.contains("`do` after with bindings"));
}

#[test]
fn with_parse_missing_else() {
    let err = parse_program("let x = with a <- 1 do a end").expect_err("missing else");
    assert!(err.message.contains("`else` clause"));
}

#[test]
fn with_resolve_rejects_undefined_in_body() {
    let program = parse_program("let x = with a <- 1 do z else 0 end").unwrap();
    let mut resolver = Resolver::new();
    let err = resolver.resolve_program(&program).expect_err("z undefined");
    assert!(matches!(err, ResolveError::Undefined(_)));
}

#[test]
fn with_resolve_else_cannot_see_with_bindings() {
    let program = parse_program("let x = with a <- 1 do a else a end").unwrap();
    let mut resolver = Resolver::new();
    let err = resolver
        .resolve_program(&program)
        .expect_err("a not visible in else");
    assert!(matches!(err, ResolveError::Undefined(_)));
}

#[test]
fn with_in_pipe_chain() {
    let env = run(
        "let safe_head = fn xs -> with [h, _] <- xs do h else 0 end end \
         let y = [42, 99] |> safe_head",
    );
    assert_eq!(env.get("y"), Some(&Value::Int(42)));
}
