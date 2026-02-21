use mictylish::ast::{Expr, Stmt};
use mictylish::lexer::lex;
use mictylish::parser::parse_program;
use mictylish::span::span;
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
        Stmt::Let {
            name,
            mutable,
            expr,
            span: stmt_span,
            ..
        } => {
            assert_eq!(name, "answer");
            assert!(!mutable);
            assert!(matches!(expr, Expr::Int(42, _)));
            assert_eq!(stmt_span, &span(0, "let answer = 42".len()));
        }
    }
}

#[test]
fn parses_mutable_let_with_precise_spans() {
    let source = "let mut items = [1, 2]";
    let program = parse_program(source).expect("parse should succeed");
    assert_eq!(program.stmts.len(), 1);
    match &program.stmts[0] {
        Stmt::Let {
            name,
            name_span,
            mutable,
            expr,
            span: stmt_span,
        } => {
            assert_eq!(name, "items");
            assert!(*mutable);
            assert_eq!(name_span, &span(8, 5));
            assert_eq!(stmt_span, &span(0, source.len()));
            match expr {
                Expr::List(items, list_span) => {
                    assert_eq!(items.len(), 2);
                    assert_eq!(list_span, &span(16, 6));
                }
                other => panic!("expected list expression, got {other:?}"),
            }
        }
    }
}

#[test]
fn reports_missing_identifier_after_let() {
    let err = parse_program("let = 1").expect_err("should fail");
    assert!(err.message.contains("expected identifier"));
    assert_eq!(err.span, span(4, 1));
}

#[test]
fn reports_missing_equal_after_binding_name() {
    let err = parse_program("let value 1").expect_err("should fail");
    assert!(err.message.contains("expected '=' after let binding"));
    assert_eq!(err.span, span(10, 1));
}

#[test]
fn reports_missing_expression_in_let() {
    let err = parse_program("let x =").expect_err("should fail");
    assert!(err.message.contains("expected expression"));
    assert_eq!(err.span, span(7, 0));
}

#[test]
fn rejects_pipe_until_t04() {
    let err = parse_program("let x = source |> sink").expect_err("pipe not ready in T03");
    assert!(
        err.message.contains("T04"),
        "actual message: {}",
        err.message
    );
}
