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
fn parses_simple_pipe_in_let() {
    let source = "let x = source |> sink";
    let program = parse_program(source).expect("parse should succeed");
    match &program.stmts[0] {
        Stmt::Let { expr, .. } => match expr {
            Expr::Pipe(lhs, rhs, pipe_span) => {
                assert!(matches!(**lhs, Expr::Var(ref n, _) if n == "source"));
                assert!(matches!(**rhs, Expr::Var(ref n, _) if n == "sink"));
                assert_eq!(pipe_span, &span(8, source.len() - 8));
            }
            other => panic!("expected Pipe, got {other:?}"),
        },
    }
}

#[test]
fn pipe_is_left_associative() {
    let source = "let x = a |> b |> c";
    let program = parse_program(source).expect("parse should succeed");
    match &program.stmts[0] {
        Stmt::Let { expr, .. } => match expr {
            Expr::Pipe(outer_lhs, outer_rhs, _) => {
                assert!(matches!(**outer_rhs, Expr::Var(ref n, _) if n == "c"));
                match &**outer_lhs {
                    Expr::Pipe(inner_lhs, inner_rhs, _) => {
                        assert!(matches!(**inner_lhs, Expr::Var(ref n, _) if n == "a"));
                        assert!(matches!(**inner_rhs, Expr::Var(ref n, _) if n == "b"));
                    }
                    other => panic!("expected nested Pipe, got {other:?}"),
                }
            }
            other => panic!("expected Pipe, got {other:?}"),
        },
    }
}

#[test]
fn pipe_inside_list_item_binds_before_comma() {
    let source = "let xs = [1, 2 |> id]";
    let program = parse_program(source).expect("parse should succeed");
    match &program.stmts[0] {
        Stmt::Let { expr, .. } => match expr {
            Expr::List(items, _) => {
                assert_eq!(items.len(), 2);
                assert!(matches!(items[0], Expr::Int(1, _)));
                match &items[1] {
                    Expr::Pipe(lhs, rhs, _) => {
                        assert!(matches!(**lhs, Expr::Int(2, _)));
                        assert!(matches!(**rhs, Expr::Var(ref n, _) if n == "id"));
                    }
                    other => panic!("expected Pipe in list, got {other:?}"),
                }
            }
            other => panic!("expected List, got {other:?}"),
        },
    }
}

#[test]
fn reports_missing_rhs_after_pipe() {
    let err = parse_program("let x = a |>").expect_err("should fail");
    assert!(err.message.contains("expected expression"));
}
