use mictylish::error::EvalError;
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

fn run_err(source: &str) -> EvalError {
    let program = parse_program(source).expect("parse");
    let mut resolver = Resolver::new();
    resolver.resolve_program(&program).expect("resolve");
    let mut env = EvalEnv::new();
    eval_program(&mut env, &program).expect_err("should fail")
}

// --- T08: io boundary ---

#[test]
fn io_block_returns_body_value() {
    let env = run("let x = io do 42 end");
    assert_eq!(env.get("x"), Some(&Value::Int(42)));
}

#[test]
fn io_block_allows_run_text() {
    let env = run(r#"let x = io do run_text("echo", "hello") end"#);
    assert_eq!(
        env.get("x"),
        Some(&Value::Ok(Box::new(Value::String("hello".to_string()))))
    );
}

#[test]
fn run_text_outside_io_is_rejected() {
    let err = run_err(r#"let x = run_text("echo", "hi")"#);
    assert!(matches!(err, EvalError::IoRequired(_)));
}

#[test]
fn nested_io_blocks_allow_run_text() {
    let env = run(r#"let x = io do io do run_text("echo", "nested") end end"#);
    assert_eq!(
        env.get("x"),
        Some(&Value::Ok(Box::new(Value::String("nested".to_string()))))
    );
}

#[test]
fn io_propagates_through_pipe() {
    let env = run(
        r#"let trim = fn x -> x end let y = io do run_text("echo", "piped") |> trim end"#,
    );
    assert_eq!(
        env.get("y"),
        Some(&Value::Ok(Box::new(Value::String("piped".to_string()))))
    );
}

#[test]
fn io_propagates_into_fn_call_via_pipe() {
    let env = run(
        r#"let get = fn prog -> run_text(prog, "from_fn") end let y = io do "echo" |> get end"#,
    );
    assert_eq!(
        env.get("y"),
        Some(&Value::Ok(Box::new(Value::String("from_fn".to_string()))))
    );
}

// --- T09: run_text now returns Result values ---

#[test]
fn run_text_command_not_found_returns_err_value() {
    let env = run(r#"let x = io do run_text("__mictylish_no_such_cmd__") end"#);
    assert!(matches!(env.get("x"), Some(Value::Err(_))));
}

#[test]
fn run_text_nonzero_exit_returns_err_value() {
    let env = run(r#"let x = io do run_text("false") end"#);
    assert!(matches!(env.get("x"), Some(Value::Err(_))));
}

#[test]
fn unknown_builtin_is_rejected() {
    let err = run_err("let x = io do no_such_builtin() end");
    assert!(matches!(err, EvalError::UnknownBuiltin(_)));
}

// --- parse errors ---

#[test]
fn parse_io_missing_do() {
    let err = parse_program("let x = io 42 end").expect_err("missing do");
    assert!(err.message.contains("`do` after `io`"));
}

#[test]
fn parse_io_missing_end() {
    let err = parse_program("let x = io do 42").expect_err("missing end");
    assert!(err.message.contains("`end` to close io block"));
}

#[test]
fn parse_call_with_multiple_args() {
    let program = parse_program(r#"let x = io do run_text("echo", "a", "b") end"#)
        .expect("parse should succeed");
    assert_eq!(program.stmts.len(), 1);
}

#[test]
fn run_text_multiple_args_concatenated() {
    let env = run(r#"let x = io do run_text("echo", "a", "b") end"#);
    assert_eq!(
        env.get("x"),
        Some(&Value::Ok(Box::new(Value::String("a b".to_string()))))
    );
}

// --- with + Result pattern ---

#[test]
fn with_ok_pattern_extracts_success() {
    let env = run(
        r#"let result = io do run_text("echo", "hi") end
           let msg = with Ok(s) <- result do s else "failed" end"#,
    );
    assert_eq!(env.get("msg"), Some(&Value::String("hi".to_string())));
}

#[test]
fn with_ok_pattern_falls_to_else_on_err() {
    let env = run(
        r#"let result = io do run_text("false") end
           let msg = with Ok(s) <- result do s else "failed" end"#,
    );
    assert_eq!(env.get("msg"), Some(&Value::String("failed".to_string())));
}
