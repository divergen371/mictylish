use mictylish::ast::Stmt;
use mictylish::error::EvalError;
use mictylish::eval::{eval_expr, eval_program, EvalEnv};
use mictylish::parser::parse_program;
use mictylish::resolver::Resolver;
use mictylish::value::Value;

#[test]
fn eval_let_binds_int() {
    let program = parse_program("let x = 7").unwrap();
    let mut resolver = Resolver::new();
    resolver.resolve_program(&program).unwrap();
    let mut env = EvalEnv::new();
    let out = eval_program(&mut env, &program).unwrap();
    assert_eq!(out, vec![("x".to_string(), Value::Int(7))]);
    assert_eq!(env.get("x"), Some(&Value::Int(7)));
}

#[test]
fn eval_pipe_identity_passes_value() {
    let program = parse_program("let x = 99 |> id").unwrap();
    let mut resolver = Resolver::new();
    resolver.resolve_program(&program).unwrap();
    let mut env = EvalEnv::new();
    let out = eval_program(&mut env, &program).unwrap();
    assert_eq!(out[0].1, Value::Int(99));
}

#[test]
fn eval_list_and_vars() {
    let program = parse_program("let a = 1 let b = [a, 2]").unwrap();
    let mut resolver = Resolver::new();
    resolver.resolve_program(&program).unwrap();
    let mut env = EvalEnv::new();
    eval_program(&mut env, &program).unwrap();
    assert_eq!(
        env.get("b"),
        Some(&Value::List(vec![Value::Int(1), Value::Int(2)]))
    );
}

#[test]
fn eval_string_literal() {
    let program = parse_program("let s = \"hi\"").unwrap();
    let mut resolver = Resolver::new();
    resolver.resolve_program(&program).unwrap();
    let mut env = EvalEnv::new();
    eval_program(&mut env, &program).unwrap();
    assert_eq!(env.get("s"), Some(&Value::String("hi".to_string())));
}

#[test]
fn eval_stores_fn_value() {
    let program = parse_program("let id = fn x -> x end").unwrap();
    let mut resolver = Resolver::new();
    resolver.resolve_program(&program).unwrap();
    let mut env = EvalEnv::new();
    eval_program(&mut env, &program).unwrap();
    assert!(matches!(
        env.get("id"),
        Some(Value::Function(func)) if func.param == "x"
    ));
}

#[test]
fn eval_pipe_calls_user_function() {
    let program = parse_program("let id = fn x -> x end let y = 5 |> id").unwrap();
    let mut resolver = Resolver::new();
    resolver.resolve_program(&program).unwrap();
    let mut env = EvalEnv::new();
    eval_program(&mut env, &program).unwrap();
    assert_eq!(env.get("y"), Some(&Value::Int(5)));
}

#[test]
fn eval_pipe_function_can_build_list() {
    let program = parse_program("let dup = fn x -> [x, x] end let y = 3 |> dup").unwrap();
    let mut resolver = Resolver::new();
    resolver.resolve_program(&program).unwrap();
    let mut env = EvalEnv::new();
    eval_program(&mut env, &program).unwrap();
    assert_eq!(
        env.get("y"),
        Some(&Value::List(vec![Value::Int(3), Value::Int(3)]))
    );
}

#[test]
fn pipe_to_bound_non_callable_name_fails_eval() {
    let program = parse_program("let identity = 1 let x = 2 |> identity").unwrap();
    let mut resolver = Resolver::new();
    resolver.resolve_program(&program).unwrap();
    let mut env = EvalEnv::new();
    let err = eval_program(&mut env, &program).expect_err("int is not a pipe target");
    assert!(matches!(err, EvalError::PipeNotCallable(_)));
}

#[test]
fn eval_fails_when_var_unbound() {
    let program = parse_program("let x = y").unwrap();
    let Stmt::Let { expr, .. } = &program.stmts[0] else {
        panic!("expected Let");
    };
    let env = EvalEnv::new();
    assert!(eval_expr(&env, expr).is_err());
}
