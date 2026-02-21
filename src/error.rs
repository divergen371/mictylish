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
