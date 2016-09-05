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

use position::Pos;

/// Struct which holds information about an error at a specific position.
#[derive(Debug, PartialEq)]
pub struct Error {
    expected: String,
    pos: Pos,
    unexpected: String,
}

impl Error {
    /// Create a new error.
    pub fn new(unexpected: String, expected: String, pos: Pos) -> Error {
        Error {
            expected: expected,
            pos: pos,
            unexpected: unexpected,
        }
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        write!(formatter, "unexpected {}, expecting {} on {}", self.unexpected, self.expected, self.pos)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "parse error"
    }
}
