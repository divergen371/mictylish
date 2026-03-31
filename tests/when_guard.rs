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
fn when_guard_selects_arm() {
    let env = run("let x = match 5 do n when n == 5 -> 1 _ -> 0 end");
    assert_eq!(env.get("x"), Some(&Value::Int(1)));
}

#[test]
fn when_guard_skips_to_next_arm() {
    let env = run("let x = match 3 do n when n == 5 -> 1 _ -> 0 end");
    assert_eq!(env.get("x"), Some(&Value::Int(0)));
}

#[test]
fn when_guard_with_not_equal() {
    let env = run(r#"let x = match "hi" do s when s != "bye" -> 1 _ -> 0 end"#);
    assert_eq!(env.get("x"), Some(&Value::Int(1)));
}

#[test]
fn when_guard_uses_pattern_binding() {
    let env = run("let target = 10 let x = match 10 do n when n == target -> n _ -> 0 end");
    assert_eq!(env.get("x"), Some(&Value::Int(10)));
}

#[test]
fn when_guard_with_list_pattern() {
    let env = run(
        "let x = match [1, 2] do [a, b] when a == b -> 0 [a, b] when a != b -> [a, b] _ -> [] end",
    );
    assert_eq!(
        env.get("x"),
        Some(&Value::List(vec![Value::Int(1), Value::Int(2)]))
    );
}

#[test]
fn when_guard_false_falls_through_to_wildcard() {
    let env = run("let x = match 0 do n when n == 1 -> 10 n when n == 2 -> 20 _ -> 99 end");
    assert_eq!(env.get("x"), Some(&Value::Int(99)));
}

#[test]
fn multiple_guarded_arms() {
    let env = run(
        "let classify = fn x -> match x do \
           n when n == 0 -> \"zero\" \
           n when n == 1 -> \"one\" \
           _ -> \"other\" \
         end end \
         let a = 0 |> classify \
         let b = 1 |> classify \
         let c = 9 |> classify",
    );
    assert_eq!(env.get("a"), Some(&Value::String("zero".to_string())));
    assert_eq!(env.get("b"), Some(&Value::String("one".to_string())));
    assert_eq!(env.get("c"), Some(&Value::String("other".to_string())));
}

#[test]
fn eq_and_neq_as_standalone_expr() {
    let env = run("let a = 1 == 1 let b = 1 != 2 let c = 1 == 2");
    assert_eq!(env.get("a"), Some(&Value::Bool(true)));
    assert_eq!(env.get("b"), Some(&Value::Bool(true)));
    assert_eq!(env.get("c"), Some(&Value::Bool(false)));
}

#[test]
fn string_equality() {
    let env = run(r#"let x = "hello" == "hello" let y = "a" != "b""#);
    assert_eq!(env.get("x"), Some(&Value::Bool(true)));
    assert_eq!(env.get("y"), Some(&Value::Bool(true)));
}

#[test]
fn guard_resolver_rejects_undefined_in_guard() {
    let program = parse_program("let x = match 1 do n when z == 1 -> n _ -> 0 end").unwrap();
    let mut resolver = Resolver::new();
    let err = resolver.resolve_program(&program).expect_err("z undefined");
    assert!(matches!(err, ResolveError::Undefined(_)));
}

#[test]
fn guard_resolver_allows_pattern_var_in_guard() {
    let program = parse_program("let x = match 1 do n when n == 1 -> n _ -> 0 end").unwrap();
    let mut resolver = Resolver::new();
    resolver.resolve_program(&program).expect("n is bound by pattern");
}
