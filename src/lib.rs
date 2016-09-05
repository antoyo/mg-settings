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

/*
 * TODO: Add map command. A map command is a command that ends with map and the prefix is the mode.
 * TODO: Add unmap command.
 * TODO: Add include command.
 * TODO: Add array type.
 */

#![warn(missing_docs)]

pub mod error;
#[doc(hidden)]
pub mod position;

use std::io::BufRead;

use error::Error;
use position::Pos;

use Command::*;
use Value::*;

/// The `Command` enum represents a command from a config file.
#[derive(Debug, PartialEq)]
pub enum Command {
    /// A set command sets a value to an option.
    Set(String, Value),
}

/// A type alias over the specific `Result` type used by the parser to indicate whether it is
/// successful or not.
pub type Result<T> = std::result::Result<T, Box<std::error::Error>>;

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
pub fn parse<R: BufRead>(input: R) -> Result<Vec<Command>> {
    let mut commands = vec![];
    for (line_num, input_line) in input.lines().enumerate() {
        if let Some(command) = try!(line(&try!(input_line), line_num as u32 + 1)) {
            commands.push(command);
        }
    }
    Ok(commands)
}

/// Check if a string is an identifier.
fn check_ident(string: String, pos: &Pos) -> Result<String> {
    if string.chars().all(|character| character.is_alphanumeric() || character == '-' || character == '_') {
        if let Some(true) = string.chars().next().map(|character| character.is_alphabetic()) {
            return Ok(string)
        }
    }
    Err(Box::new(Error::new(string, "identifier".to_string(), pos.clone())))
}

/// Parse a line.
fn line(line: &str, line_num: u32) -> Result<Option<Command>> {
    if let Some(word) = word(line) {
        if word.starts_with('#') {
            Ok(None)
        }
        else {
            match word {
                "set" => set_command(&line[4..], line_num, 5).map(Some),
                _ => {
                    let index = line.find(word).unwrap() as u32 + 1; // NOTE: the word is in the line, hence unwrap.
                    Err(Box::new(Error::new(word.to_string(), "command or comment".to_string(), Pos::new(line_num, index))))
                },
            }
        }
    }
    else {
        Ok(None)
    }
}

/// Parse a set command.
fn set_command(line: &str, line_num: u32, column_num: u32) -> Result<Command> {
    if let Some(words) = words(line, 2) {
        let index = line.find(words[0]).unwrap() as u32; // NOTE: the line contains the word, hence unwrap.
        let identifier = try!(check_ident(words[0].to_string(), &Pos::new(line_num, column_num + index)));
        let operator = words[1];
        if operator == "=" {
            let index = line.find('=').unwrap(); // NOTE: the line contains an =, hence unwrap.
            let rest = &line[index + 1..];
            Ok(Set(identifier.to_string(), try!(value(rest))))
        }
        else {
            let index = line.find(operator).unwrap() as u32; // NOTE: the operator is in the line, hence unwrap.
            Err(Box::new(Error::new(operator.to_string(), "=".to_string(), Pos::new(line_num, column_num + index))))
        }
    }
    else {
        Err(Box::new(Error::new("<eof>".to_string(), "identifier".to_string(), Pos::new(line_num, column_num))))
    }
}

/// Parse a value.
fn value(input: &str) -> Result<Value> {
    let string: String = input.chars().take_while(|&character| character != '#').collect();
    let string = string.trim();
    match string {
        "true" => Ok(Bool(true)),
        "false" => Ok(Bool(false)),
        _ => {
            if string.chars().all(|character| character.is_digit(10)) {
                Ok(Int(string.parse().unwrap())) // NOTE: the string only contains digit, hence unwrap.
            }
            else if string.chars().all(|character| character.is_digit(10) || character == '.') {
                Ok(Float(string.parse().unwrap())) // NOTE: the string only contains digit or dot, hence unwrap.
            }
            else {
                Ok(Str(input.trim().to_string()))
            }
        },
    }
}

/// Parse a single word.
fn word(input: &str) -> Option<&str> {
    input.split_whitespace().next()
}

/// Parse a `count` words.
fn words(input: &str, count: usize) -> Option<Vec<&str>> {
    let vec: Vec<_> = input.split_whitespace().take(count).collect();
    if vec.len() == count {
        Some(vec)
    }
    else {
        None
    }
}
