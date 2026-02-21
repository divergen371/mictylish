use crate::ast::{Expr, Program, Stmt};
use crate::error::ParseError;
use crate::lexer::lex;
use crate::token::{Token, TokenKind};

pub fn parse_program(source: &str) -> Result<Program, ParseError> {
    let tokens = lex(source)?;
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

#[derive(Debug, Clone)]
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut stmts = Vec::new();
        while !self.is_eof() {
            stmts.push(self.parse_stmt()?);
        }
        Ok(Program::new(stmts))
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek_kind() {
            TokenKind::Let => self.parse_let_stmt(),
            other => Err(ParseError::new(
                format!("unsupported statement in T02 scaffold: {other:?}"),
                self.peek().span.clone(),
            )),
        }
    }

    fn parse_let_stmt(&mut self) -> Result<Stmt, ParseError> {
        let let_token = self.bump();
        let name = self.expect_ident()?;
        self.expect(TokenKind::Equal, "expected '=' after let binding")?;
        let expr = self.parse_expr()?;
        Ok(Stmt::Let {
            name,
            expr,
            span: let_token.span,
        })
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_primary()?;
        if self.matches(&TokenKind::PipeGreater) {
            return Err(ParseError::new(
                "`|>` parsing is planned for T04",
                self.peek().span.clone(),
            ));
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let token = self.bump();
        match token.kind {
            TokenKind::Int(v) => Ok(Expr::Int(v, token.span)),
            TokenKind::String(v) => Ok(Expr::String(v, token.span)),
            TokenKind::Ident(name) => Ok(Expr::Var(name, token.span)),
            TokenKind::LBracket => self.parse_list(token.span),
            TokenKind::Fn | TokenKind::Match | TokenKind::With | TokenKind::Io => {
                Err(ParseError::new(
                    "this construct is reserved for later tasks",
                    token.span,
                ))
            }
            other => Err(ParseError::new(
                format!("expected expression, found {other:?}"),
                token.span,
            )),
        }
    }

    fn parse_list(&mut self, start_span: miette::SourceSpan) -> Result<Expr, ParseError> {
        let mut items = Vec::new();
        if self.matches(&TokenKind::RBracket) {
            self.bump();
            return Ok(Expr::List(items, start_span));
        }
        loop {
            items.push(self.parse_expr()?);
            if self.matches(&TokenKind::Comma) {
                self.bump();
                continue;
            }
            self.expect(TokenKind::RBracket, "expected ']' to close list")?;
            break;
        }
        Ok(Expr::List(items, start_span))
    }

    fn expect(&mut self, expected: TokenKind, message: &str) -> Result<(), ParseError> {
        if self.matches(&expected) {
            self.bump();
            Ok(())
        } else {
            Err(ParseError::new(message, self.peek().span.clone()))
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        let token = self.bump();
        match token.kind {
            TokenKind::Ident(name) => Ok(name),
            _ => Err(ParseError::new(
                "expected identifier",
                token.span.clone(),
            )),
        }
    }

    fn matches(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(kind)
    }

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .unwrap_or_else(|| self.tokens.last().expect("tokens always contains EOF"))
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    fn bump(&mut self) -> Token {
        let token = self.peek().clone();
        if !self.is_eof() {
            self.pos += 1;
        }
        token
    }

    fn is_eof(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Eof)
    }
}
