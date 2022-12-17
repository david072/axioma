/*
 * Copyright (c) 2022, david072
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use std::ops::Range;
use std::path::PathBuf;
use rust_decimal::Decimal;
use thiserror::Error;
use crate::FromPrimitive;

const CRATE_NAME: &str = "funcially";

#[derive(Error, Debug)]
pub enum ErrorType {
    /// Not actually an error. Used when e.g.
    /// a variable needs a value, but will never be used.
    #[error("")]
    Nothing,
    // tokenizer
    #[error("Invalid Character {0}")]
    InvalidCharacter(String),
    #[error("Could not parse number")]
    InvalidNumber,

    // parser
    #[error("Expected Number")]
    ExpectedNumber,
    #[error("Expected Operator")]
    ExpectedOperator,
    #[error("Expected 'in'")]
    ExpectedIn,
    #[error("Expected a format (hex/binary/decimal)")]
    ExpectedFormat,
    #[error("Missing opening bracket")]
    MissingOpeningBracket,
    #[error("Missing closing bracket")]
    MissingClosingBracket,
    #[error("Expected an identifier")]
    ExpectedIdentifier,
    #[error("Unknown Identifier \"{0}\"")]
    UnknownIdentifier(String),
    #[error("Unknown Variable \"{0}\"")]
    UnknownVariable(String),
    #[error("Missing equals sign")]
    MissingEqualsSign,
    #[error("Equals signs are only allowed at the top level")]
    UnexpectedEqualsSign,
    #[error("Second equals sign")]
    UnexpectedSecondEqualsSign,
    #[error("Unknown function \"{0}\"")]
    UnknownFunction(String),
    #[error("Wrong number of arguments (expected {0} arguments)")]
    WrongNumberOfArguments(usize),
    #[error("Wrong number of arguments (expected one of {0:?} arguments)")]
    WrongNumberOfArgumentsMultiple(&'static [usize]),
    #[error("Expected unit")]
    ExpectedUnit,
    #[error("Unexpected unit")]
    UnexpectedUnit,
    #[error("Expected text")]
    ExpectedElements,
    #[error("Expected end")]
    UnexpectedElements,
    #[error("Unexpected comma")]
    UnexpectedComma,
    #[error("Definitions are only allowed at the top level")]
    UnexpectedDefinition,
    #[error("Expected expression, found {0}")]
    ExpectedExpression(String),
    #[error("Expected opening bracket")]
    ExpectedOpenBracket,
    #[error("Expected closing bracket")]
    ExpectedCloseBracket,
    #[error("Expected comma")]
    ExpectedComma,
    #[error("Can't redefine standard variable \"{0}\"")]
    ReservedVariable(String),
    #[error("Can't redefine standard function \"{0}\"")]
    ReservedFunction(String),
    #[error("Argument \"{0}\" already given")]
    DuplicateArgument(String),
    #[error("Unexpected question mark")]
    UnexpectedQuestionMark,
    #[error("A question mark is not allowed here")]
    QuestionMarkNotAllowed,
    #[error("Can't use what's being defined")]
    CantUseIdentifierInDefinition,

    // engine
    #[error("Cannot divide by zero")]
    DivideByZero,
    #[error("Expected integer for operator '{0}'")]
    ExpectedIntegerWithOperator(String),
    #[error("Expected an integer, found {0}")]
    ExpectedInteger(f64),
    #[error("Expected percentage for 'of' operator")]
    ExpectedPercentage,
    #[error("Argument 1 must be less than argument 2")]
    Arg1GreaterThanArg2,
    #[error("Unknown conversion ({0} -> {1})")]
    UnknownConversion(String, String),
    #[error("Not a number")]
    NotANumber,
    #[error("Powers can't be used around the unknown variable")]
    ForbiddenExponentiation,
    #[error("Expected unknown variable (?)")]
    ExpectedQuestionMark,
    #[error("Wrong unit, expected {0}")]
    WrongUnit(String),
    /// This should never happen
    #[error("")]
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

pub fn round_dp(n: f64, dp: u32) -> String {
    if n.is_nan() { return "NaN".to_owned(); }
    if !n.is_finite() { return "infinity".to_owned(); }
    match Decimal::from_f64(n) {
        Some(decimal) => decimal.round_dp(dp).to_string(),
        None => n.to_string(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn cache_dir() -> PathBuf { dirs::cache_dir().unwrap().join(CRATE_NAME) }

pub fn data_dir() -> PathBuf { dirs::data_local_dir().unwrap().join(CRATE_NAME) }

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