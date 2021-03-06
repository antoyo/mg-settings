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

//! Settings error type.

use std::fmt::{self, Display, Formatter};

use self::SettingError::{UnknownChoice, UnknownSetting, WrongType};

/// Error when getting/setting settings.
#[allow(missing_docs)]
#[derive(Debug, PartialEq)]
pub enum SettingError {
    /// Unknown setting value choice.
    UnknownChoice {
        // The actual value.
        actual: String,
        // The list of expected values.
        expected: Vec<&'static str>
    },
    /// Unknown setting name.
    UnknownSetting(String),
    /// Wrong value type for setting.
    WrongType {
        // The actual type.
        actual: String,
        // The expected type.
        expected: String,
    },
}

impl Display for SettingError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match *self {
            UnknownChoice { ref actual, ref expected } =>
                write!(formatter, "unknown choice {}, expecting one of: {}", actual, expected.join(", ")),
            UnknownSetting(ref name) =>
                write!(formatter, "no setting named {}", name),
            WrongType { ref actual, ref expected } =>
                write!(formatter, "wrong value type: expecting {}, but found {}", expected, actual),
        }
    }
}
