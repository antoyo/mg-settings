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

//! Parse config files.
//!
//! # Usage
//!
//! Call the `parse` function on the input.

#![warn(missing_docs)]

pub mod error;
#[doc(hidden)]
pub mod position;
#[doc(hidden)]
pub mod tokens;

use error::Error;
use position::{Pos, WithPos};
use tokens::{Token, Tokens, tokenize};
use tokens::Token::Eof;

use Command::*;
use Value::*;

macro_rules! peek {
    ($tokens:expr, { $($patterns:pat => $block:expr),* $(,)* }) => {
        match $tokens.peek() {
            $(Some(&Ok(WithPos { node: $patterns, .. })) => $block,)*
            _ => Ok(None),
        }
    };
}

macro_rules! switch {
    ($tokens:expr, $expected:expr, { $($($patterns:pat)|* => $block:expr),* $(,)* }) => {
        match $tokens.next() {
            Some(Ok(WithPos { node: Eof, pos })) => return Err(Error::new(Eof.to_string(), $expected, pos)),
            $($(Some(Ok(WithPos { node: $patterns, .. })))|* => $block,)*
            Some(Err(error)) => return Err(error),
            None => unreachable!(),
        }
    };
    ($tokens:expr, $expected:expr, $pos:ident, { $($($patterns:pat)|* => $block:expr),* $(,)* }) => {
        match $tokens.next() {
            Some(Ok(WithPos { node: Eof, pos })) => return Err(Error::new(Eof.to_string(), $expected, pos)),
            $($(Some(Ok(WithPos { node: $patterns, pos: $pos })))|* => $block,)*
            Some(Err(error)) => return Err(error),
            None => unreachable!(),
        }
    };
}

/// The `Command` enum represents a command from a config file.
#[derive(Debug, PartialEq)]
pub enum Command {
    /// A set command sets a value to an option.
    Set(String, Value),
}

/// A type alias over the specific `Result` type used by the parser to indicate whether it is
/// successful or not.
pub type Result<T> = std::result::Result<T, Error>;

/// The `Value` enum represents a value along with its type.
#[derive(Debug, PartialEq)]
pub enum Value {
    /// Boolean value.
    Bool(bool),
    /// Floating-point value.
    Float(f64),
    /// Integer value.
    Int(i64),
    /// String value.
    Str(String),
}

/// Parse settings.
pub fn parse(input: &str) -> Result<Vec<Command>> {
    let mut tokens = tokenize(input);
    let mut commands = vec![];
    while let Some(command) = try!(line(&mut tokens)) {
        commands.push(command);
    }
    try!(eof(&mut tokens));
    Ok(commands)
}

/// Check if a string is an identifier.
fn check_ident(string: String, pos: &Pos) -> Result<String> {
    if string.chars().all(|character| character.is_alphanumeric() || character == '-' || character == '_') {
        Ok(string)
    }
    else {
        Err(Error::new(string, "identifier".into(), pos.clone()))
    }
}

/// Parse end of file.
fn eof(tokens: &mut Tokens) -> Result<()> {
    match tokens.next() {
        Some(Ok(WithPos { node: Eof, .. })) => Ok(()),
        Some(Ok(token)) => Err(Error::new(token.to_string(), "command or comment".into(), token.pos)),
        Some(Err(error)) => Err(error),
        None => unreachable!(),
    }
}

/// Parse a line.
fn line(tokens: &mut Tokens) -> Result<Option<Command>> {
    peek!(tokens, {
        Token::Set => set_command(tokens).map(Some),
    })
}

/// Parse a set command.
fn set_command(tokens: &mut Tokens) -> Result<Command> {
    tokens.next();
    let expected = "identifier".to_string();
    let variable =
        switch!(tokens, expected, pos, {
            Token::Str(variable) => try!(check_ident(variable, &pos)),
            token => return Err(Error::new(token.to_string(), expected, pos.clone())),
        });
    let value = try!(value(tokens));
    Ok(Set(variable, value))
}

/// Parse a value.
fn value(tokens: &mut Tokens) -> Result<Value> {
    let expected = "value".to_string();
    let val =
        switch!(tokens, expected, pos, {
            Token::Bool(boolean) => Bool(boolean),
            Token::Float(num) => Float(num),
            Token::Int(num) => Int(num),
            Token::Str(string) | Token::QuotedStr(string) => Str(string),
            token => return Err(Error::new(token.to_string(), expected, pos.clone())),
        });
    Ok(val)
}
