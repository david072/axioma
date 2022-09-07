/*
 * Copyright (c) 2022, david072
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use std::ops::Range;
use eframe::egui::Color32;
use crate::astgen::tokenizer::{Token, TokenType};
use self::TokenType::*;

#[derive(Debug, Clone)]
pub struct ColorSegment {
    pub range: Range<usize>,
    pub color: Color32,
}

impl ColorSegment {
    pub fn new(range: Range<usize>, color: Color32) -> Self {
        Self { range, color }
    }

    pub fn from(token: &Token) -> ColorSegment {
        let color = match token.ty {
            Whitespace => Color32::TRANSPARENT,
            DecimalLiteral | HexLiteral | BinaryLiteral => Color32::KHAKI,
            OpenBracket | CloseBracket => Color32::WHITE,
            Plus | Minus | Multiply | Divide | Exponentiation | BitwiseAnd | BitwiseOr | Of | In => Color32::GOLD,
            ExclamationMark | PercentSign => Color32::WHITE,
            Decimal | Hex | Binary | Identifier => Color32::LIGHT_BLUE,
            Comma | EqualsSign | DefinitionSign => Color32::WHITE,
        };
        ColorSegment {
            range: token.range.clone(),
            color,
        }
    }
}