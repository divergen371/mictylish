use mictylish::ast::{Expr, Stmt};
use mictylish::lexer::lex;
use mictylish::parser::parse_program;
use mictylish::token::TokenKind;

#[test]
fn lexes_let_binding_tokens() {
    let tokens = lex("let x = 1").expect("lex should succeed");
    assert_eq!(
        tokens.iter().map(|t| &t.kind).collect::<Vec<_>>(),
        vec![
            &TokenKind::Let,
            &TokenKind::Ident("x".to_string()),
            &TokenKind::Equal,
            &TokenKind::Int(1),
            &TokenKind::Eof
        ]
    );
}

#[test]
fn parses_single_let_statement() {
    let program = parse_program("let answer = 42").expect("parse should succeed");
    assert_eq!(program.stmts.len(), 1);
    match &program.stmts[0] {
        Stmt::Let { name, expr, .. } => {
            assert_eq!(name, "answer");
            assert!(matches!(expr, Expr::Int(42, _)));
        }
    }
}

#[test]
fn rejects_pipe_until_t04() {
    let err = parse_program("let x = source |> sink").expect_err("pipe not ready in T02");
    assert!(
        err.message.contains("T04"),
        "actual message: {}",
        err.message
    );
}
