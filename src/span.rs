use miette::{SourceOffset, SourceSpan};

pub type Span = SourceSpan;

pub fn span(start: usize, len: usize) -> Span {
    SourceSpan::new(SourceOffset::from(start), len)
}
