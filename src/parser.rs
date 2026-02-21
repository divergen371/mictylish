use crate::ast::{Expr, Program, Stmt};
use crate::error::ParseError;
use crate::lexer::lex;
use crate::span::covering;
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
            _ => Err(self.expected_error("statement")),
        }
    }

    fn parse_let_stmt(&mut self) -> Result<Stmt, ParseError> {
        let let_token = self.bump();
        let mutable = if self.matches(&TokenKind::Mut) {
            self.bump();
            true
        } else {
            false
        };
        let (name, name_span) = self.expect_ident()?;
        self.expect(TokenKind::Equal, "'=' after let binding")?;
        let expr = self.parse_expr()?;
        let span = covering(&let_token.span, &expr.span());
        Ok(Stmt::Let {
            name,
            name_span,
            mutable,
            expr,
            span,
        })
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_primary()?;
        if self.matches(&TokenKind::PipeGreater) {
            return Err(ParseError::new(
                "pipe operator '|>' is not available yet (planned for T04)",
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
                Err(ParseError::new("expected expression, found reserved keyword", token.span))
            }
            _ => Err(ParseError::new(
                format!("expected expression, found {}", token_label(&token.kind)),
                token.span,
            )),
        }
    }

    fn parse_list(&mut self, start_span: miette::SourceSpan) -> Result<Expr, ParseError> {
        let mut items = Vec::new();
        if self.matches(&TokenKind::RBracket) {
            let end = self.bump();
            return Ok(Expr::List(items, covering(&start_span, &end.span)));
        }
        loop {
            items.push(self.parse_expr()?);
            if self.matches(&TokenKind::Comma) {
                self.bump();
                continue;
            }
            let end = self.expect(TokenKind::RBracket, "']' to close list")?;
            let span = covering(&start_span, &end.span);
            return Ok(Expr::List(items, span));
        }
    }

    fn expect(&mut self, expected: TokenKind, expected_desc: &str) -> Result<Token, ParseError> {
        if self.matches(&expected) {
            Ok(self.bump())
        } else {
            Err(self.expected_error(expected_desc))
        }
    }

    fn expect_ident(&mut self) -> Result<(String, miette::SourceSpan), ParseError> {
        let token = self.bump();
        match token.kind {
            TokenKind::Ident(name) => Ok((name, token.span)),
            _ => Err(ParseError::new(
                format!("expected identifier, found {}", token_label(&token.kind)),
                token.span,
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

    fn expected_error(&self, expected: &str) -> ParseError {
        ParseError::new(
            format!("expected {expected}, found {}", token_label(self.peek_kind())),
            self.peek().span.clone(),
        )
    }
}

fn token_label(kind: &TokenKind) -> &'static str {
    match kind {
        TokenKind::Let => "`let`",
        TokenKind::Mut => "`mut`",
        TokenKind::Set => "`set`",
        TokenKind::Fn => "`fn`",
        TokenKind::Match => "`match`",
        TokenKind::With => "`with`",
        TokenKind::When => "`when`",
        TokenKind::Io => "`io`",
        TokenKind::Do => "`do`",
        TokenKind::End => "`end`",
        TokenKind::Ident(_) => "identifier",
        TokenKind::Int(_) => "integer literal",
        TokenKind::String(_) => "string literal",
        TokenKind::PipeGreater => "`|>`",
        TokenKind::Arrow => "`->`",
        TokenKind::LeftArrow => "`<-`",
        TokenKind::Equal => "`=`",
        TokenKind::Comma => "`,`",
        TokenKind::LParen => "`(`",
        TokenKind::RParen => "`)`",
        TokenKind::LBracket => "`[`",
        TokenKind::RBracket => "`]`",
        TokenKind::Eof => "end of input",
    }
}
