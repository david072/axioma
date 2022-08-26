/*
 * Copyright (c) 2022, david072
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use std::ops::Range;

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorType {
    /// Not actually an error. Used when e.g.
    /// a variable needs a value, but will never be used.
    Nothing,
    // tokenizer
    InvalidCharacter,
    InvalidNumber,
    UnknownWord,

    // parser
    ExpectedNumber,
    ExpectedOperator,
    ExpectedIn,
    ExpectedFormat,
    MissingOpeningBracket,
    MissingClosingBracket,
    UnknownIdentifier,
    UnknownVariable,
    UnexpectedEqualsSign,
    UnexpectedSecondEqualsSign,
    UnknownFunction,
    WrongNumberOfArguments,
    UnexpectedUnit,
    ExpectedElements,

    // engine
    DivideByZero,
    ExpectedInteger,
    ExpectedPositiveInteger,
    ExpectedPercentage,
    InvalidArguments,
    UnknownConversion,
    NotANumber,
    /// This should never happen
    InvalidAst,
}

impl ErrorType {
    pub fn with(self, range: Range<usize>) -> Error {
        Error {
            error: self,
            start: range.start,
            end: range.end,
        }
    }
}

#[derive(Debug)]
pub struct Error {
    pub error: ErrorType,
    pub start: usize,
    pub end: usize,
}

pub type Result<T> = std::result::Result<T, Error>;

pub mod math {
    pub fn factorial(num: i64) -> i64 {
        match num {
            0 => 1,
            1 => 1,
            _ => {
                let factor = if num.is_negative() { -1 } else { 1 };
                factor * factorial(num.abs() - 1) * num
            }
        }
    }
}