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
pub enum Pattern {
    Wildcard(Span),
    Int(i64, Span),
    String(String, Span),
    Var(String, Span),
    List(Vec<Pattern>, Span),
}

impl Pattern {
    pub fn span(&self) -> Span {
        match self {
            Pattern::Wildcard(s) => *s,
            Pattern::Int(_, s) => *s,
            Pattern::String(_, s) => *s,
            Pattern::Var(_, s) => *s,
            Pattern::List(_, s) => *s,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    String(String, Span),
    Int(i64, Span),
    Var(String, Span),
    List(Vec<Expr>, Span),
    Fn {
        param: String,
        param_span: Span,
        body: Box<Expr>,
        span: Span,
    },
    Match {
        subject: Box<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    /// `with pat <- expr, ... do body else fallback end`
    With {
        bindings: Vec<WithBinding>,
        body: Box<Expr>,
        else_body: Box<Expr>,
        span: Span,
    },
    /// Left-associative pipeline: `a |> b |> c` is `((a |> b) |> c)`.
    Pipe(Box<Expr>, Box<Expr>, Span),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WithBinding {
    pub pattern: Pattern,
    pub expr: Expr,
    pub span: Span,
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::String(_, s) => s.clone(),
            Expr::Int(_, s) => s.clone(),
            Expr::Var(_, s) => s.clone(),
            Expr::List(_, s) => s.clone(),
            Expr::Fn { span, .. } => *span,
            Expr::Match { span, .. } => *span,
            Expr::With { span, .. } => *span,
            Expr::Pipe(_, _, s) => s.clone(),
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
