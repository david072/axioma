/*
 * Copyright (c) 2022, david072
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::common::*;
use strum::EnumIter;
use std::ops::Range;

#[derive(Debug, EnumIter, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Whitespace,
    // Literals
    DecimalLiteral,
    HexLiteral,
    BinaryLiteral,
    // Brackets
    OpenBracket,
    CloseBracket,
    // Operators
    Plus,
    Minus,
    Multiply,
    Divide,
    Exponentiation,
    BitwiseAnd,
    BitwiseOr,
    BitShiftLeft,
    BitShiftRight,
    Of,
    In,
    Modulo,
    // Modifiers
    ExclamationMark,
    PercentSign,
    // Formats
    Decimal,
    Hex,
    Binary,
    Scientific,
    // Identifier
    Identifier,
    Comma,
    EqualsSign,
    DefinitionSign,
    QuestionMark,
}

impl TokenType {
    pub fn is_literal(&self) -> bool {
        matches!(self, Self::DecimalLiteral
            | Self::HexLiteral
            | Self::BinaryLiteral
            | Self::QuestionMark) // '?' has the same rules a literal
    }

    pub fn is_number(&self) -> bool {
        self.is_literal() || matches!(self, Self::OpenBracket
            | Self::CloseBracket
            | Self::Identifier)
    }

    pub fn is_operator(&self) -> bool {
        matches!(self, Self::Plus
            | Self::Minus
            | Self::Multiply
            | Self::Divide
            | Self::Exponentiation
            | Self::BitwiseAnd
            | Self::BitwiseOr
            | Self::BitShiftLeft
            | Self::BitShiftRight
            | Self::Of
            | Self::In
            | Self::Modulo
            | Self::EqualsSign) // '=' has the same rules as an operator
    }

    pub fn is_format(&self) -> bool {
        matches!(self, Self::Decimal | Self::Hex | Self::Binary | Self::Scientific)
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Token {
    pub ty: TokenType,
    pub text: String,
    pub range: Range<usize>,
}

pub fn tokenize(input: &str) -> Result<Vec<Token>> {
    let mut tokenizer = Tokenizer::new(input);
    let mut result = Vec::new();

    while let Some(token) = tokenizer.next()? {
        match token.ty {
            TokenType::Whitespace => continue,
            _ => result.push(token),
        }
    }

    Ok(result)
}

const NUMBERS: &str = "0123456789_";
const LETTERS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";
const HEXADECIMAL_CHARS: &str = "0123456789abcdefABCDEF_";
const BINARY_DIGITS: &str = "01_";
const WHITESPACE: &str = " \t\r\n";

fn any_of(chars: &str) -> impl Fn(u8) -> bool + '_ {
    move |c| chars.contains(c as char)
}

struct Tokenizer<'a> {
    source: &'a str,
    string: &'a [u8],
    index: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Tokenizer {
        Tokenizer {
            source,
            string: source.as_bytes(),
            index: 0,
        }
    }

    pub fn next(&mut self) -> Result<Option<Token>> {
        if self.index >= self.string.len() {
            return Ok(None);
        }

        let start = self.index;
        let next_ty = self.next_type();
        let end = self.index;

        match next_ty {
            Some(mut ty) => {
                let slice = self.string[start..end].to_owned();
                let slice = match String::from_utf8(slice) {
                    Ok(v) => v,
                    Err(e) => panic!("Failed to parse string '{:?}' ({}..{} in {:?}) ({})",
                                     &self.string[start..end], start, end, self.string, e),
                };

                if ty == TokenType::Identifier {
                    ty = match slice.to_lowercase().as_str() {
                        "of" => TokenType::Of,
                        "in" => TokenType::In,
                        "mod" => TokenType::Modulo,
                        "decimal" => TokenType::Decimal,
                        "hex" => TokenType::Hex,
                        "binary" => TokenType::Binary,
                        "scientific" | "sci" => TokenType::Scientific,
                        _ => ty,
                    };
                }

                Ok(Some(Token {
                    ty,
                    text: slice,
                    range: start..std::cmp::max(0, end),
                }))
            }
            None => {
                // Move end to a char boundary
                let mut end = end;
                while !self.source.is_char_boundary(end) {
                    end += 1;
                }

                Err(ErrorType::InvalidCharacter(
                    String::from_utf8(self.string[start..end].to_owned()).unwrap_or_default()
                ).with(start..end))
            }
        }
    }

    fn accept<F: Fn(u8) -> bool>(&mut self, predicate: F) -> bool {
        if self.index >= self.string.len() {
            return false;
        }

        if predicate(self.string[self.index]) {
            self.index += 1;
            return true;
        }

        false
    }

    fn is_next(&mut self, char: u8) -> bool {
        if let Some(c) = self.string.get(self.index) {
            if *c == char {
                self.index += 1;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn next_type(&mut self) -> Option<TokenType> {
        if self.accept(any_of(WHITESPACE)) {
            while self.accept(any_of(WHITESPACE)) {}
            return Some(TokenType::Whitespace);
        }

        let c = self.string[self.index];
        self.index += 1;
        let res = match c {
            b'0'..=b'9' => {
                if c == b'0' && self.index < self.string.len() {
                    // check next character for different representation
                    let c = self.string[self.index];
                    self.index += 1;
                    match c {
                        b'x' | b'X' => {
                            while self.accept(any_of(HEXADECIMAL_CHARS)) {}
                            return Some(TokenType::HexLiteral);
                        }
                        b'b' | b'B' => {
                            while self.accept(any_of(BINARY_DIGITS)) {}
                            return Some(TokenType::BinaryLiteral);
                        }
                        // fall through to after the if
                        b'0'..=b'9' | b'.' => {}
                        _ => {
                            // the character needs to be processed in the next iteration
                            self.index -= 1;
                            return Some(TokenType::DecimalLiteral);
                        }
                    }
                }

                while self.accept(any_of(NUMBERS)) {}
                self.accept(any_of("."));
                while self.accept(any_of(NUMBERS)) {}
                Some(TokenType::DecimalLiteral)
            }
            b'.' => {
                while self.accept(any_of(NUMBERS)) {}
                Some(TokenType::DecimalLiteral)
            }
            b'+' => Some(TokenType::Plus),
            b'-' => Some(TokenType::Minus),
            b'*' => Some(TokenType::Multiply),
            b'/' => Some(TokenType::Divide),
            b'^' => Some(TokenType::Exponentiation),
            b'&' => Some(TokenType::BitwiseAnd),
            b'|' => Some(TokenType::BitwiseOr),
            b'<' if self.is_next(b'<') => Some(TokenType::BitShiftLeft),
            b'>' if self.is_next(b'>') => Some(TokenType::BitShiftRight),
            b'!' => Some(TokenType::ExclamationMark),
            b'%' => Some(TokenType::PercentSign),
            b'(' => Some(TokenType::OpenBracket),
            b')' => Some(TokenType::CloseBracket),
            b'=' => Some(TokenType::EqualsSign),
            b',' => Some(TokenType::Comma),
            b':' if self.is_next(b'=') => Some(TokenType::DefinitionSign),
            b'?' => Some(TokenType::QuestionMark),
            _ => None
        };

        if res.is_some() { return res; }

        if c == 0xC2 { // First byte of "°"
            if self.is_next(0xB0) { // Second byte of "°"
                while self.accept(any_of(LETTERS)) {}
                Some(TokenType::Identifier)
            } else {
                None
            }
        } else if LETTERS.contains(c as char) {
            let mut iterations = 0usize;
            while self.accept(any_of(LETTERS)) { iterations += 1; }
            // Necessary for scientific notation (need 'e' and number separately)
            // e.g. in "1e2" the "e2" should result in two tokens
            if iterations == 0 &&
                (self.string[self.index - 1] == b'e' || self.string[self.index - 1] == b'E') {
                return Some(TokenType::Identifier);
            }

            while self.accept(any_of(LETTERS)) ||
                self.accept(any_of(NUMBERS)) {}
            while self.accept(any_of("'‘’`")) {}
            Some(TokenType::Identifier)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Token {
        fn new(ty: TokenType, text: &str, range: Range<usize>) -> Token {
            Token { ty, text: text.to_owned(), range }
        }
    }

    #[test]
    fn literals() -> Result<()> {
        let tokens = tokenize("3 0x0123456789 0xABCdef 0b110")?;
        assert_eq!(tokens, vec![
            Token::new(TokenType::DecimalLiteral, "3", 0..1),
            Token::new(TokenType::HexLiteral, "0x0123456789", 2..14),
            Token::new(TokenType::HexLiteral, "0xABCdef", 15..23),
            Token::new(TokenType::BinaryLiteral, "0b110", 24..29),
        ]);
        Ok(())
    }

    #[test]
    fn operators() -> Result<()> {
        let tokens = tokenize("+ - * /")?;
        assert_eq!(tokens, vec![
            Token::new(TokenType::Plus, "+", 0..1),
            Token::new(TokenType::Minus, "-", 2..3),
            Token::new(TokenType::Multiply, "*", 4..5),
            Token::new(TokenType::Divide, "/", 6..7),
        ]);
        Ok(())
    }

    #[test]
    fn extended_operators() -> Result<()> {
        let tokens = tokenize("^ & | ! of % =")?;
        assert_eq!(tokens, vec![
            Token::new(TokenType::Exponentiation, "^", 0..1),
            Token::new(TokenType::BitwiseAnd, "&", 2..3),
            Token::new(TokenType::BitwiseOr, "|", 4..5),
            Token::new(TokenType::ExclamationMark, "!", 6..7),
            Token::new(TokenType::Of, "of", 8..10),
            Token::new(TokenType::PercentSign, "%", 11..12),
            Token::new(TokenType::EqualsSign, "=", 13..14),
        ]);
        Ok(())
    }

    #[test]
    fn floats() -> Result<()> {
        let tokens = tokenize("0.23 .23")?;
        assert_eq!(tokens, vec![
            Token::new(TokenType::DecimalLiteral, "0.23", 0..4),
            Token::new(TokenType::DecimalLiteral, ".23", 5..8),
        ]);
        Ok(())
    }

    #[test]
    fn groups() -> Result<()> {
        let tokens = tokenize("()")?;
        assert_eq!(tokens, vec![
            Token::new(TokenType::OpenBracket, "(", 0..1),
            Token::new(TokenType::CloseBracket, ")", 1..2),
        ]);
        Ok(())
    }

    #[test]
    fn identifiers() -> Result<()> {
        let tokens = tokenize("1 test tset")?;
        assert_eq!(tokens[1..], vec![
            Token::new(TokenType::Identifier, "test", 2..6),
            Token::new(TokenType::Identifier, "tset", 7..11),
        ]);
        Ok(())
    }

    #[test]
    fn non_ascii_chars() -> Result<()> {
        let tokens = tokenize("°")?;
        assert_eq!(tokens, vec![Token::new(TokenType::Identifier, "°", 0..2)]);
        Ok(())
    }
}