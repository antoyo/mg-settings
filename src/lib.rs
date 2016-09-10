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
 * TODO: Add unmap command.
 * TODO: Add array type.
 */

#![warn(missing_docs)]

pub mod error;
pub mod key;
#[macro_use]
mod macros;
#[doc(hidden)]
pub mod position;

use std::collections::HashMap;
use std::io::BufRead;

use error::{Error, Result};
use key::{Key, parse_keys};
use position::Pos;

use Command::*;
use Value::*;

/// The `Command` enum represents a command from a config file.
#[derive(Debug, PartialEq)]
pub enum Command {
    /// An include command includes another configuration file.
    Include(String),
    /// A map command creates a new key mapping.
    Map {
        /// The action that will be executed when the `keys` are pressed.
        action: String,
        /// The key shortcut to trigger the action.
        keys: Vec<Key>,
        /// The mode in which this mapping is available.
        mode: String,
    },
    /// A set command sets a value to an option.
    Set(String, Value),
}

/// The parsing configuration.
#[derive(Default)]
pub struct Config {
    /// The available mapping modes for the map command.
    pub mapping_modes: Vec<String>,
}

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
    parse_with_config(input, Config::default())
}

/// Parse settings.
pub fn parse_with_config<R: BufRead>(input: R, config: Config) -> Result<Vec<Command>> {
    let mut commands = vec![];
    for (line_num, input_line) in input.lines().enumerate() {
        if let Some(command) = try!(line(&try!(input_line), line_num + 1, &config)) {
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

/// Parse an include command.
fn include_command(line: &str, _line_num: usize, _column_num: usize) -> Result<Command> {
    Ok(Include(word(line).to_string()))
}

/// Parse a line.
fn line(line: &str, line_num: usize, config: &Config) -> Result<Option<Command>> {
    let include_func = &include_command;
    let set_func = &set_command;
    let commands =
        hash! {<&str, &Fn(&str, usize, usize) -> Result<Command> >
            "include" => include_func,
            "set" => set_func,
        };

    if let Some(word) = maybe_word(line) {
        let start_index = word.len() + 1;
        let column = start_index + 1;

        let command_with_args = |command: &Fn(&str, usize, usize) -> Result<Command>| {
            if line.len() > start_index {
                command(&line[start_index..], line_num, column).map(Some)
            }
            else {
                Err(From::from(Box::new(Error::new(
                    "<end of line>".to_string(),
                    "command arguments".to_string(),
                    Pos::new(line_num, start_index)
                ))))
            }
        };

        let (start, end) =
            if word.len() > 3 {
                word.split_at(word.len() - 3)
            }
            else {
                ("", "")
            };
        if word.starts_with('#') {
            Ok(None)
        }
        else if let Some(command) = commands.get(word) {
            command_with_args(command)
        }
        else if end == "map" && config.mapping_modes.contains(&start.to_string()) {
            command_with_args(&|line, line_num, column_num| map_command(line, line_num, column_num, start))
        }
        else {
            // NOTE: the word is in the line, hence unwrap.
            let index = line.find(word).unwrap() + 1;
            Err(Box::new(Error::new(
                word.to_string(),
                "command or comment".to_string(),
                Pos::new(line_num, index)
            )))
        }
    }
    else {
        Ok(None)
    }
}

/// Parse a map command.
fn map_command(line: &str, line_num: usize, column_num: usize, mode: &str) -> Result<Command> {
    let word = word(line);

    // NOTE: the line contains the word, hence unwrap.
    let index = line.find(word).unwrap();

    let rest = &line[index + word.len() ..].trim();
    if !rest.is_empty() {
        Ok(Map {
            action: rest.to_string(),
            keys: try!(parse_keys(word, line_num, column_num + index)),
            mode: mode.to_string(),
        })
    }
    else {
        Err(Box::new(Error::new(
            "<end of line>".to_string(),
            "mapping action".to_string(),
            Pos::new(line_num, column_num + line.len())
        )))
    }
}

/// Parse a single word.
fn maybe_word(input: &str) -> Option<&str> {
    input.split_whitespace()
        .next()
}

/// Parse a set command.
fn set_command(line: &str, line_num: usize, column_num: usize) -> Result<Command> {
    if let Some(words) = words(line, 2) {
        // NOTE: the line contains the word, hence unwrap.
        let index = line.find(words[0]).unwrap();
        let identifier = try!(check_ident(words[0].to_string(), &Pos::new(line_num, column_num + index)));

        let operator = words[1];
        // NOTE: the operator is in the line, hence unwrap.
        let operator_index = line.find(operator).unwrap();
        if operator == "=" {
            let rest = &line[operator_index + 1..];
            Ok(Set(identifier.to_string(), try!(value(rest, line_num, column_num + operator_index + 1))))
        }
        else {
            Err(Box::new(Error::new(
                operator.to_string(),
                "=".to_string(),
                Pos::new(line_num, column_num + operator_index)
            )))
        }
    }
    else {
        Err(Box::new(Error::new(
            "<end of line>".to_string(),
            "=".to_string(),
            Pos::new(line_num, column_num + line.len())
        )))
    }
}

/// Parse a value.
fn value(input: &str, line_num: usize, column_num: usize) -> Result<Value> {
    let string: String = input.chars().take_while(|&character| character != '#').collect();
    let string = string.trim();
    match string {
        "" => Err(Box::new(Error::new(
                  "<end of line>".to_string(),
                  "value".to_string(),
                  Pos::new(line_num, column_num + string.len())
              ))),
        "true" => Ok(Bool(true)),
        "false" => Ok(Bool(false)),
        _ => {
            if string.chars().all(|character| character.is_digit(10)) {
                // NOTE: the string only contains digit, hence unwrap.
                Ok(Int(string.parse().unwrap()))
            }
            else if string.chars().all(|character| character.is_digit(10) || character == '.') {
                // NOTE: the string only contains digit or dot, hence unwrap.
                Ok(Float(string.parse().unwrap()))
            }
            else {
                Ok(Str(input.trim().to_string()))
            }
        },
    }
}

/// Parse a single word.
/// This function assumes there is always at least a word in `input`.
fn word(input: &str) -> &str {
    input.split_whitespace()
        .next()
        .unwrap()
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
