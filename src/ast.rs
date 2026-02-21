use crate::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

impl Program {
    pub fn new(stmts: Vec<Stmt>) -> Self {
        Self { stmts }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        name_span: Span,
        mutable: bool,
        expr: Expr,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    String(String, Span),
    Int(i64, Span),
    Var(String, Span),
    List(Vec<Expr>, Span),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::String(_, s) => s.clone(),
            Expr::Int(_, s) => s.clone(),
            Expr::Var(_, s) => s.clone(),
            Expr::List(_, s) => s.clone(),
        }
    }
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::Let { span, .. } => span.clone(),
        }
    }
}
