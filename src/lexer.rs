use std::iter::Peekable;
use std::str::CharIndices;

use crate::error::ParseError;
use crate::span::span;
use crate::token::{Token, TokenKind};

pub fn lex(source: &str) -> Result<Vec<Token>, ParseError> {
    let mut lexer = Lexer {
        source,
        chars: source.char_indices().peekable(),
    };
    lexer.lex_tokens()
}

struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<CharIndices<'a>>,
}

impl<'a> Lexer<'a> {
    fn lex_tokens(&mut self) -> Result<Vec<Token>, ParseError> {
        let mut tokens = Vec::new();

        while let Some((idx, ch)) = self.chars.peek().cloned() {
            if ch.is_whitespace() {
                self.chars.next();
                continue;
            }

            if is_ident_start(ch) {
                tokens.push(self.lex_ident_or_keyword(idx));
                continue;
            }

            if ch.is_ascii_digit() {
                tokens.push(self.lex_int(idx)?);
                continue;
            }

            match ch {
                '"' => tokens.push(self.lex_string(idx)?),
                '|' => {
                    self.chars.next();
                    self.expect_char('>', idx, "expected '>' after '|'")?;
                    tokens.push(Token::new(TokenKind::PipeGreater, span(idx, 2)));
                }
                '-' => {
                    self.chars.next();
                    self.expect_char('>', idx, "expected '>' after '-'")?;
                    tokens.push(Token::new(TokenKind::Arrow, span(idx, 2)));
                }
                '<' => {
                    self.chars.next();
                    self.expect_char('-', idx, "expected '-' after '<'")?;
                    tokens.push(Token::new(TokenKind::LeftArrow, span(idx, 2)));
                }
                '=' => {
                    self.chars.next();
                    tokens.push(Token::new(TokenKind::Equal, span(idx, 1)));
                }
                ',' => {
                    self.chars.next();
                    tokens.push(Token::new(TokenKind::Comma, span(idx, 1)));
                }
                '(' => {
                    self.chars.next();
                    tokens.push(Token::new(TokenKind::LParen, span(idx, 1)));
                }
                ')' => {
                    self.chars.next();
                    tokens.push(Token::new(TokenKind::RParen, span(idx, 1)));
                }
                '[' => {
                    self.chars.next();
                    tokens.push(Token::new(TokenKind::LBracket, span(idx, 1)));
                }
                ']' => {
                    self.chars.next();
                    tokens.push(Token::new(TokenKind::RBracket, span(idx, 1)));
                }
                _ => {
                    return Err(ParseError::new(
                        format!("unexpected character '{ch}'"),
                        span(idx, ch.len_utf8()),
                    ));
                }
            }
        }

        tokens.push(Token::new(TokenKind::Eof, span(self.source.len(), 0)));
        Ok(tokens)
    }

    fn lex_ident_or_keyword(&mut self, start: usize) -> Token {
        let mut end = start;
        while let Some((idx, ch)) = self.chars.peek().cloned() {
            if is_ident_continue(ch) {
                end = idx + ch.len_utf8();
                self.chars.next();
            } else {
                break;
            }
        }
        let text = &self.source[start..end];
        let kind = match text {
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "set" => TokenKind::Set,
            "fn" => TokenKind::Fn,
            "match" => TokenKind::Match,
            "with" => TokenKind::With,
            "when" => TokenKind::When,
            "io" => TokenKind::Io,
            "do" => TokenKind::Do,
            "end" => TokenKind::End,
            _ => TokenKind::Ident(text.to_string()),
        };
        Token::new(kind, span(start, end.saturating_sub(start)))
    }

    fn lex_int(&mut self, start: usize) -> Result<Token, ParseError> {
        let mut end = start;
        while let Some((idx, ch)) = self.chars.peek().cloned() {
            if ch.is_ascii_digit() {
                end = idx + ch.len_utf8();
                self.chars.next();
            } else {
                break;
            }
        }
        let text = &self.source[start..end];
        let value = text.parse::<i64>().map_err(|_| {
            ParseError::new(
                format!("invalid integer literal '{text}'"),
                span(start, end.saturating_sub(start)),
            )
        })?;
        Ok(Token::new(
            TokenKind::Int(value),
            span(start, end.saturating_sub(start)),
        ))
    }

    fn lex_string(&mut self, start: usize) -> Result<Token, ParseError> {
        self.chars.next();
        let mut content = String::new();
        let mut end = start + 1;

        while let Some((idx, ch)) = self.chars.next() {
            if ch == '"' {
                end = idx + 1;
                return Ok(Token::new(
                    TokenKind::String(content),
                    span(start, end.saturating_sub(start)),
                ));
            }
            content.push(ch);
            end = idx + ch.len_utf8();
        }

        Err(ParseError::new(
            "unterminated string literal",
            span(start, end.saturating_sub(start)),
        ))
    }

    fn expect_char(
        &mut self,
        expected: char,
        start: usize,
        message: &str,
    ) -> Result<(), ParseError> {
        match self.chars.next() {
            Some((_, ch)) if ch == expected => Ok(()),
            Some((idx, ch)) => Err(ParseError::new(
                format!("{message}, found '{ch}'"),
                span(idx, ch.len_utf8()),
            )),
            None => Err(ParseError::new(message, span(start, 1))),
        }
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}
