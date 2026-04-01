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
fn ok_wraps_value() {
    let env = run("let x = ok(42)");
    assert_eq!(env.get("x"), Some(&Value::Ok(Box::new(Value::Int(42)))));
}

#[test]
fn err_wraps_value() {
    let env = run(r#"let x = err("bad")"#);
    assert_eq!(
        env.get("x"),
        Some(&Value::Err(Box::new(Value::String("bad".to_string()))))
    );
}

#[test]
fn is_ok_true_for_ok() {
    let env = run("let x = is_ok(ok(1))");
    assert_eq!(env.get("x"), Some(&Value::Bool(true)));
}

#[test]
fn is_ok_false_for_err() {
    let env = run(r#"let x = is_ok(err("nope"))"#);
    assert_eq!(env.get("x"), Some(&Value::Bool(false)));
}

#[test]
fn is_err_true_for_err() {
    let env = run(r#"let x = is_err(err("nope"))"#);
    assert_eq!(env.get("x"), Some(&Value::Bool(true)));
}

#[test]
fn is_err_false_for_ok() {
    let env = run("let x = is_err(ok(1))");
    assert_eq!(env.get("x"), Some(&Value::Bool(false)));
}

#[test]
fn match_ok_pattern() {
    let env = run("let x = match ok(5) do Ok(n) -> n _ -> 0 end");
    assert_eq!(env.get("x"), Some(&Value::Int(5)));
}

#[test]
fn match_err_pattern() {
    let env = run(r#"let x = match err("bad") do Ok(n) -> n Err(e) -> e end"#);
    assert_eq!(env.get("x"), Some(&Value::String("bad".to_string())));
}

#[test]
fn match_ok_does_not_match_err() {
    let env = run(r#"let x = match err("x") do Ok(n) -> n _ -> 0 end"#);
    assert_eq!(env.get("x"), Some(&Value::Int(0)));
}

#[test]
fn with_ok_extracts_value() {
    let env = run("let x = with Ok(v) <- ok(10) do v else 0 end");
    assert_eq!(env.get("x"), Some(&Value::Int(10)));
}

#[test]
fn with_ok_falls_through_on_err() {
    let env = run(r#"let x = with Ok(v) <- err("bad") do v else 0 end"#);
    assert_eq!(env.get("x"), Some(&Value::Int(0)));
}

#[test]
fn nested_ok_in_list() {
    let env = run("let x = [ok(1), ok(2), err(3)]");
    assert_eq!(
        env.get("x"),
        Some(&Value::List(vec![
            Value::Ok(Box::new(Value::Int(1))),
            Value::Ok(Box::new(Value::Int(2))),
            Value::Err(Box::new(Value::Int(3))),
        ]))
    );
}

#[test]
fn ok_err_equality() {
    let env = run("let a = ok(1) == ok(1) let b = ok(1) == ok(2) let c = ok(1) == err(1)");
    assert_eq!(env.get("a"), Some(&Value::Bool(true)));
    assert_eq!(env.get("b"), Some(&Value::Bool(false)));
    assert_eq!(env.get("c"), Some(&Value::Bool(false)));
}

#[test]
fn ok_err_display() {
    let env = run("let x = ok(42)");
    assert_eq!(format!("{}", env.get("x").unwrap()), "Ok(42)");
    let env2 = run(r#"let x = err("bad")"#);
    assert_eq!(format!("{}", env2.get("x").unwrap()), r#"Err("bad")"#);
}

#[test]
fn pipe_through_fn_returning_result() {
    let env = run(
        "let safe_div = fn x -> match x do \
           0 -> err(\"division by zero\") \
           n -> ok(100) \
         end end \
         let a = 5 |> safe_div \
         let b = 0 |> safe_div",
    );
    assert_eq!(env.get("a"), Some(&Value::Ok(Box::new(Value::Int(100)))));
    assert!(matches!(env.get("b"), Some(Value::Err(_))));
}
