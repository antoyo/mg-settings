/*
 * Copyright (c) 2017 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

//! Parse and io error type.

pub mod settings;

use position::Pos;
pub use self::settings::SettingError;

error_chain! {
    errors {
        /// Parse error.
        Parse(error: ParseError) {
            description("parse error")
            display("unexpected {}, expecting {} on {}", error.unexpected, error.expected, error.pos)
        }
        /// Error when getting/setting settings.
        Setting(error: SettingError) {
            description(error.description())
            display("{}", error)
        }
    }

    foreign_links {
        Io(::std::io::Error) /// Input/output error.
        ;
    }
}

/// A set of error types that can occur parsing the settings file.
#[derive(Debug, PartialEq)]
pub enum ErrorType {
    /// A missing argument.
    MissingArgument,
    /// No command (or a comment) was entered.
    NoCommand,
    /// Parse error.
    Parse,
    /// Unknown command.
    UnknownCommand,
}

/// Struct which holds information about an error at a specific position.
#[derive(Debug, PartialEq)]
pub struct ParseError {
    /// The expected token.
    pub expected: String,
    pos: Pos,
    /// The error type.
    pub typ: ErrorType,
    /// The unexpected token.
    pub unexpected: String,
}

impl ParseError {
    /// Create a new error.
    pub fn new(typ: ErrorType, unexpected: String, expected: String, pos: Pos) -> ParseError {
        ParseError {
            expected: expected,
            pos: pos,
            typ: typ,
            unexpected: unexpected,
        }
    }
}
