use miette::{SourceOffset, SourceSpan};

pub type Span = SourceSpan;

pub fn span(start: usize, len: usize) -> Span {
    SourceSpan::new(SourceOffset::from(start), len)
}

pub fn covering(start: &Span, end: &Span) -> Span {
    let begin = start.offset().min(end.offset());
    let finish = (start.offset() + start.len()).max(end.offset() + end.len());
    span(begin, finish.saturating_sub(begin))
}
