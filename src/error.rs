/*
 * Copyright (c) 2016 Boucher, Antoni <bouanto@zoho.com>
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

//! Parse error type.

use std::error;
use std::fmt::{self, Display, Formatter};
use std::result;

use position::Pos;
use self::SettingError::*;

/// Struct which holds information about an error at a specific position.
#[derive(Debug, PartialEq)]
pub struct Error {
    /// The expected token.
    pub expected: String,
    pos: Pos,
    /// The error type.
    pub typ: ErrorType,
    /// The unexpected token.
    pub unexpected: String,
}

impl Error {
    /// Create a new error.
    pub fn new(typ: ErrorType, unexpected: String, expected: String, pos: Pos) -> Error {
        Error {
            expected: expected,
            pos: pos,
            typ: typ,
            unexpected: unexpected,
        }
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> ::std::result::Result<(), fmt::Error> {
        write!(formatter, "unexpected {}, expecting {} on {}", self.unexpected, self.expected, self.pos)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "parse error"
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

/// Error when getting/setting settings.
#[derive(Debug)]
pub enum SettingError {
    /// Unknown setting value choice.
    UnknownChoice {
        /// The actual value.
        actual: String,
        /// The list of expected values.
        expected: Vec<&'static str>,
    },
    /// Unknown setting name.
    UnknownSetting(String),
    /// Wrong value type for setting.
    WrongType {
        /// The actual type.
        actual: String,
        /// The expected type.
        expected: String,
    },
}

impl Display for SettingError {
    fn fmt(&self, formatter: &mut Formatter) -> result::Result<(), fmt::Error> {
        match *self {
            UnknownChoice { ref actual, ref expected } => {
                let expected = expected.join(", ");
                write!(formatter, "unknown choice {}, expecting one of: {}", actual, expected)
            },
            UnknownSetting(ref name) => write!(formatter, "no setting named {}", name),
            WrongType { ref actual, ref expected } => write!(formatter, "wrong value type: expecting {}, but found {}", expected, actual),
        }
    }
}

impl error::Error for SettingError {
    fn description(&self) -> &str {
        match *self {
            UnknownChoice { .. } => "unknown choice",
            UnknownSetting(_) => "unknown setting name",
            WrongType { .. } => "wrong value type",
        }
    }
}

/// A type alias over the specific `Result` type used by the parser to indicate whether it is
/// successful or not.
pub type Result<T> = ::std::result::Result<T, Box<::std::error::Error>>;
