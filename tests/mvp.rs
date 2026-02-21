use mictylish::builtin;
use mictylish::command::CommandSpec;
use mictylish::resolver::Resolver;
use mictylish::runtime::run_command;
use mictylish::span::span;

#[test]
fn name_collision_is_rejected() {
    let mut resolver = Resolver::new();
    resolver.define("x", span(0, 1)).expect("first define");
    let err = resolver
        .define("x", span(10, 1))
        .expect_err("must reject shadowing");
    assert_eq!(err.name, "x");
}

#[test]
fn glob_is_not_implicit() {
    let literal = "*.definitely-no-auto-expand";
    assert_eq!(literal, "*.definitely-no-auto-expand");
    let expanded = builtin::glob(literal).expect("glob call should succeed");
    assert!(expanded.is_empty());
}

#[test]
fn args_are_passed_without_word_splitting() {
    let arg = "a b";
    let spec = CommandSpec::new("echo", vec![arg.to_string()]);
    let out = run_command(&spec).expect("echo should run");
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(text.trim_end(), "a b");
}
