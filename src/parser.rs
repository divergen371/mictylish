use crate::ast::{BinOp, Expr, MatchArm, Pattern, Program, Stmt, WithBinding};
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
            TokenKind::Set => self.parse_set_stmt(),
            _ => Ok(Stmt::Expr(self.parse_expr()?)),
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

    fn parse_set_stmt(&mut self) -> Result<Stmt, ParseError> {
        let set_token = self.bump();
        let (name, name_span) = self.expect_ident()?;
        self.expect(TokenKind::Equal, "'=' after set binding")?;
        let expr = self.parse_expr()?;
        let span = covering(&set_token.span, &expr.span());
        Ok(Stmt::Set {
            name,
            name_span,
            expr,
            span,
        })
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_comparison()?;
        while self.matches(&TokenKind::PipeGreater) {
            self.bump();
            let rhs = self.parse_comparison()?;
            let span = covering(&lhs.span(), &rhs.span());
            lhs = Expr::Pipe(Box::new(lhs), Box::new(rhs), span);
        }
        Ok(lhs)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let lhs = self.parse_primary()?;
        let op = match self.peek_kind() {
            TokenKind::EqualEqual => Some(BinOp::Eq),
            TokenKind::NotEqual => Some(BinOp::NotEq),
            _ => None,
        };
        if let Some(op) = op {
            self.bump();
            let rhs = self.parse_primary()?;
            let span = covering(&lhs.span(), &rhs.span());
            Ok(Expr::BinOp {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                span,
            })
        } else {
            Ok(lhs)
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let token = self.bump();
        match token.kind {
            TokenKind::Int(v) => Ok(Expr::Int(v, token.span)),
            TokenKind::String(v) => Ok(Expr::String(v, token.span)),
            TokenKind::Ident(name) if self.matches(&TokenKind::LParen) => {
                self.parse_call_expr(name, token.span)
            }
            TokenKind::Ident(name) => Ok(Expr::Var(name, token.span)),
            TokenKind::LBracket => self.parse_list(token.span),
            TokenKind::Fn => self.parse_fn_expr(token.span),
            TokenKind::Match => self.parse_match_expr(token.span),
            TokenKind::With => self.parse_with_expr(token.span),
            TokenKind::Io => self.parse_io_expr(token.span),
            _ => Err(ParseError::new(
                format!("expected expression, found {}", token_label(&token.kind)),
                token.span,
            )),
        }
    }

    fn parse_fn_expr(&mut self, fn_span: miette::SourceSpan) -> Result<Expr, ParseError> {
        let (param, param_span) = self.expect_ident()?;
        self.expect(TokenKind::Arrow, "'->' after function parameter")?;
        let body = self.parse_expr()?;
        let end = self.expect(TokenKind::End, "`end` to close function")?;
        Ok(Expr::Fn {
            param,
            param_span,
            body: Box::new(body),
            span: covering(&fn_span, &end.span),
        })
    }

    fn parse_match_expr(&mut self, match_span: miette::SourceSpan) -> Result<Expr, ParseError> {
        let subject = self.parse_expr()?;
        self.expect(TokenKind::Do, "`do` after match subject")?;
        let mut arms = Vec::new();
        while !self.matches(&TokenKind::End) && !self.is_eof() {
            let pattern = self.parse_pattern()?;
            let guard = if self.matches(&TokenKind::When) {
                self.bump();
                Some(self.parse_expr()?)
            } else {
                None
            };
            self.expect(TokenKind::Arrow, "'->' after match pattern")?;
            let body = self.parse_expr()?;
            let span = covering(&pattern.span(), &body.span());
            arms.push(MatchArm {
                pattern,
                guard,
                body,
                span,
            });
        }
        if arms.is_empty() {
            return Err(ParseError::new(
                "match expression must have at least one arm",
                self.peek().span.clone(),
            ));
        }
        let end = self.expect(TokenKind::End, "`end` to close match")?;
        Ok(Expr::Match {
            subject: Box::new(subject),
            arms,
            span: covering(&match_span, &end.span),
        })
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        let token = self.bump();
        match token.kind {
            TokenKind::Int(v) => Ok(Pattern::Int(v, token.span)),
            TokenKind::String(v) => Ok(Pattern::String(v, token.span)),
            TokenKind::Ident(ref name) if name == "_" => Ok(Pattern::Wildcard(token.span)),
            TokenKind::Ident(name) => Ok(Pattern::Var(name, token.span)),
            TokenKind::LBracket => self.parse_list_pattern(token.span),
            _ => Err(ParseError::new(
                format!("expected pattern, found {}", token_label(&token.kind)),
                token.span,
            )),
        }
    }

    fn parse_list_pattern(&mut self, start_span: miette::SourceSpan) -> Result<Pattern, ParseError> {
        let mut items = Vec::new();
        if self.matches(&TokenKind::RBracket) {
            let end = self.bump();
            return Ok(Pattern::List(items, covering(&start_span, &end.span)));
        }
        loop {
            items.push(self.parse_pattern()?);
            if self.matches(&TokenKind::Comma) {
                self.bump();
                continue;
            }
            let end = self.expect(TokenKind::RBracket, "']' to close list pattern")?;
            return Ok(Pattern::List(items, covering(&start_span, &end.span)));
        }
    }

    fn parse_with_expr(&mut self, with_span: miette::SourceSpan) -> Result<Expr, ParseError> {
        let mut bindings = Vec::new();
        loop {
            let pattern = self.parse_pattern()?;
            self.expect(TokenKind::LeftArrow, "'<-' after with pattern")?;
            let expr = self.parse_expr()?;
            let span = covering(&pattern.span(), &expr.span());
            bindings.push(WithBinding { pattern, expr, span });
            if self.matches(&TokenKind::Comma) {
                self.bump();
                continue;
            }
            break;
        }
        if bindings.is_empty() {
            return Err(ParseError::new(
                "with expression must have at least one binding",
                self.peek().span.clone(),
            ));
        }
        self.expect(TokenKind::Do, "`do` after with bindings")?;
        let body = self.parse_expr()?;
        let else_kw = self.peek().clone();
        if !matches!(else_kw.kind, TokenKind::Ident(ref s) if s == "else") {
            return Err(self.expected_error("`else` clause in with expression"));
        }
        self.bump();
        let else_body = self.parse_expr()?;
        let end = self.expect(TokenKind::End, "`end` to close with expression")?;
        Ok(Expr::With {
            bindings,
            body: Box::new(body),
            else_body: Box::new(else_body),
            span: covering(&with_span, &end.span),
        })
    }

    fn parse_io_expr(&mut self, io_span: miette::SourceSpan) -> Result<Expr, ParseError> {
        self.expect(TokenKind::Do, "`do` after `io`")?;
        let body = self.parse_expr()?;
        let end = self.expect(TokenKind::End, "`end` to close io block")?;
        Ok(Expr::Io {
            body: Box::new(body),
            span: covering(&io_span, &end.span),
        })
    }

    fn parse_call_expr(
        &mut self,
        name: String,
        name_span: miette::SourceSpan,
    ) -> Result<Expr, ParseError> {
        self.bump(); // consume LParen
        let mut args = Vec::new();
        if !self.matches(&TokenKind::RParen) {
            loop {
                args.push(self.parse_expr()?);
                if self.matches(&TokenKind::Comma) {
                    self.bump();
                    continue;
                }
                break;
            }
        }
        let end = self.expect(TokenKind::RParen, "')' to close function call")?;
        Ok(Expr::Call {
            name,
            name_span,
            args,
            span: covering(&name_span, &end.span),
        })
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
        TokenKind::EqualEqual => "`==`",
        TokenKind::NotEqual => "`!=`",
        TokenKind::Comma => "`,`",
        TokenKind::LParen => "`(`",
        TokenKind::RParen => "`)`",
        TokenKind::LBracket => "`[`",
        TokenKind::RBracket => "`]`",
        TokenKind::Eof => "end of input",
    }
}
