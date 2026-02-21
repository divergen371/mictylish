use crate::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Let,
    Mut,
    Set,
    Fn,
    Match,
    With,
    When,
    Io,
    Do,
    End,
    Ident(String),
    Int(i64),
    String(String),
    PipeGreater,
    Arrow,
    LeftArrow,
    Equal,
    Comma,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Eof,
}
