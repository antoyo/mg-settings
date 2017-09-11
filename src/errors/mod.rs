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

use std::fmt::{self, Display, Formatter};
use std::io;
use std::result;

use position::Pos;
pub use self::settings::SettingError;
use self::Error::{Msg, Parse, Setting};

/// Parser result type.
pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, PartialEq)]
/// Parser or setting error.
pub enum Error {
    /// Other errors like input/output error.
    Msg(String),
    /// Parse error.
    Parse(ParseError),
    /// Error when getting/setting settings.
    Setting(SettingError),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match *self {
            Msg(ref msg) => write!(formatter, "{}", msg),
            Parse(ref error) => write!(formatter, "{}", error),
            Setting(ref error) => write!(formatter, "{}", error),
        }
    }
}

impl Into<Error> for io::Error {
    fn into(self) -> Error {
        Msg(self.to_string())
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
    #[allow(unknown_lints, new_ret_no_self)]
    pub fn new(typ: ErrorType, unexpected: String, expected: String, pos: Pos) -> Error {
        Error::Parse(ParseError {
            expected: expected,
            pos: pos,
            typ: typ,
            unexpected: unexpected,
        })
    }
}

impl Display for ParseError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "unexpected {}, expecting {} on {}", self.unexpected, self.expected, self.pos)
    }
}
