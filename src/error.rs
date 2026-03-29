use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
#[error("name '{name}' already defined")]
#[diagnostic(code(mictylish::name_shadowing))]
pub struct NameError {
    pub name: String,
    #[label("previous definition here")]
    pub first: SourceSpan,
    #[label("redefined here")]
    pub second: SourceSpan,
}

#[derive(Debug, Error, Diagnostic, Clone)]
#[error("{message}")]
#[diagnostic(code(mictylish::parse_error))]
pub struct ParseError {
    pub message: String,
    #[label("parse error here")]
    pub span: SourceSpan,
}

impl ParseError {
    pub fn new(message: impl Into<String>, span: SourceSpan) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}

#[derive(Debug, Error, Diagnostic, Clone)]
#[error("name '{name}' is not defined")]
#[diagnostic(code(mictylish::undefined_name))]
pub struct UndefinedNameError {
    pub name: String,
    #[label("used here")]
    pub span: SourceSpan,
}

#[derive(Debug, Error, Diagnostic, Clone)]
#[error("pipeline right-hand side must be an identifier")]
#[diagnostic(code(mictylish::invalid_pipe_rhs))]
pub struct InvalidPipeRhsError {
    #[label("here")]
    pub span: SourceSpan,
}

#[derive(Debug, Error, Diagnostic)]
pub enum ResolveError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Shadowing(#[from] NameError),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Undefined(#[from] UndefinedNameError),
    #[error(transparent)]
    #[diagnostic(transparent)]
    InvalidPipeRhs(#[from] InvalidPipeRhsError),
}

#[derive(Debug, Error, Diagnostic)]
#[error("name '{name}' has no value in this environment")]
#[diagnostic(code(mictylish::eval_unbound))]
pub struct EvalUnboundError {
    pub name: String,
    #[label("here")]
    pub span: SourceSpan,
}

#[derive(Debug, Error, Diagnostic)]
#[error("pipeline target '{name}' is not callable")]
#[diagnostic(code(mictylish::eval_pipe_callable))]
pub struct EvalPipeNotCallableError {
    pub name: String,
    #[label("here")]
    pub span: SourceSpan,
}

#[derive(Debug, Error, Diagnostic)]
#[error("pipeline right-hand side must be an identifier")]
#[diagnostic(code(mictylish::eval_invalid_pipe_rhs))]
pub struct EvalInvalidPipeRhsError {
    #[label("here")]
    pub span: SourceSpan,
}

#[derive(Debug, Error, Diagnostic)]
#[error("no matching arm found")]
#[diagnostic(code(mictylish::eval_match_exhausted))]
pub struct EvalMatchExhaustedError {
    #[label("this value was not matched")]
    pub span: SourceSpan,
}

#[derive(Debug, Error, Diagnostic)]
pub enum EvalError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Unbound(#[from] EvalUnboundError),
    #[error(transparent)]
    #[diagnostic(transparent)]
    PipeNotCallable(#[from] EvalPipeNotCallableError),
    #[error(transparent)]
    #[diagnostic(transparent)]
    InvalidPipeRhs(#[from] EvalInvalidPipeRhsError),
    #[error(transparent)]
    #[diagnostic(transparent)]
    MatchExhausted(#[from] EvalMatchExhaustedError),
}
