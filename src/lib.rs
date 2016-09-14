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
 * TODO: create a new method instead of using words and find/unwrap.
 * TODO: Add array type.
 */

#![warn(missing_docs)]

pub mod error;
pub mod key;
#[macro_use]
mod macros;
#[doc(hidden)]
pub mod position;
mod string;

use std::io::BufRead;
use std::marker::PhantomData;

use error::{Error, Result};
use key::{Key, parse_keys};
use position::Pos;
use string::StrExt;

use Command::*;
use Value::*;

/// The `EnumFromStr` trait is used to specify how to construct an enum value from a string.
pub trait EnumFromStr
    where Self: Sized
{
    /// Create the enum value from the `variant` string and an `argument` string.
    fn create(variant: &str, argument: &str) -> std::result::Result<Self, String>;

    /// Check wether the enum variant has an argument.
    fn has_argument(variant: &str) -> std::result::Result<bool, String>;
}

#[macro_export]
macro_rules! commands {
    ($name:ident { $($command:ident $(($param:ident))*),* $(,)* }) => {
        #[derive(Debug, PartialEq)]
        pub enum $name {
            $($command $(($param))*),*
        }

        impl mg_settings::EnumFromStr for $name {
            fn create(variant: &str, argument: &str) -> ::std::result::Result<$name, String> {
                match variant {
                    $(stringify!($command) =>
                        Ok($command $(({ let _ = $param::default(); argument.to_string() }))*)
                    ,)*
                    _ => Err(format!("unknown command {}", variant.to_lowercase())),
                }
            }

            fn has_argument(variant: &str) -> ::std::result::Result<bool, String> {
                match variant {
                    $(stringify!($command) =>
                        Ok([$({ let _ = $param::default(); true },)* false][0])
                    ,)*
                    _ => Err(format!("unknown command {}", variant.to_lowercase())),
                }
            }
        }
    };
}

/// The `Command` enum represents a command from a config file.
#[derive(Debug, PartialEq)]
pub enum Command<T> {
    /// A custom command.
    Custom(T),
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
    /// An unmap command removes a key mapping.
    Unmap {
        /// The key shortcut to remove.
        keys: Vec<Key>,
        /// The mode in which this mapping is available.
        mode: String,
    },
}

/// The parsing configuration.
#[derive(Default)]
pub struct Config {
    /// The available mapping modes for the map command.
    pub mapping_modes: Vec<String>,
}

/// The config parser.
pub struct Parser<T> {
    column: usize,
    config: Config,
    line: usize,
    _phantom: PhantomData<T>,
}

impl<T: EnumFromStr> Parser<T> {
    /// Create a new parser without config.
    pub fn new() -> Self {
        Parser {
            column: 1,
            config: Config::default(),
            line: 1,
            _phantom: PhantomData,
        }
    }

    /// Create a new parser with config.
    pub fn new_with_config(config: Config) -> Self {
        Parser {
            column: 1,
            config: config,
            line: 1,
            _phantom: PhantomData,
        }
    }

    /// Check that we reached the end of the line.
    fn check_eol(&self, line: &str, index: usize) -> Result<()> {
        if line.len() > index {
            let rest = &line[index..];
            if let Some(word) = maybe_word(rest) {
                let index = rest.find(word).unwrap(); // NOTE: the line contains the word, hence unwrap.
                return Err(Box::new(Error::new(
                    rest.to_string(),
                    "<end of line>".to_string(),
                    Pos::new(self.line, self.column + index),
                )));
            }
        }
        Ok(())
    }

    /// Parse a custom command or return an error if it does not exist.
    fn custom_command(&self, line: &str, word: &str, start_index: usize, index: usize) -> Result<Command<T>> {
        let variant = &word.capitalize();
        let args =
            if line.len() > start_index {
                line[start_index..].trim()
            }
            else if let Ok(true) = T::has_argument(variant) {
                return Err(From::from(self.missing_args(start_index)));
            }
            else {
                ""
            };
        if let Ok(command) = T::create(variant, args) {
            Ok(Custom(command))
        }
        else {
            Err(Box::new(Error::new(
                word.to_string(),
                "command or comment".to_string(),
                Pos::new(self.line, index + 1)
            )))
        }
    }

    /// Get the rest of the line, starting at `column`, returning an error if the column is greater
    /// than the line's length.
    fn get_rest<'a>(&self, line: &'a str, column: usize) -> Result<&'a str> {
        if line.len() > column {
            Ok(&line[column..])
        }
        else {
            Err(From::from(self.missing_args(column)))
        }
    }

    /// Parse a line.
    fn line(&mut self, line: &str) -> Result<Option<Command<T>>> {
        if let Some(word) = maybe_word(line) {
            // NOTE: the word is in the line, hence unwrap.
            let index = line.find(word).unwrap();
            let start_index = index + word.len() + 1;
            self.column = start_index + 1;

            let (start3, end3) = word.rsplit_at(3);
            let (start5, end5) = word.rsplit_at(5);
            if word.starts_with('#') {
                return Ok(None);
            }

            let command =
                if word == "include" {
                    let rest = try!(self.get_rest(line, start_index));
                    self.include_command(rest)
                }
                else if word == "set" {
                    let rest = try!(self.get_rest(line, start_index));
                    self.set_command(rest)
                }
                else if end3 == "map" && self.config.mapping_modes.contains(&start3.to_string()) {
                    let rest = try!(self.get_rest(line, start_index));
                    self.map_command(rest, start3)
                }
                else if end5 == "unmap" && self.config.mapping_modes.contains(&start5.to_string()) {
                    let rest = try!(self.get_rest(line, start_index));
                    self.unmap_command(rest, start5)
                }
                else {
                    self.custom_command(line, word, start_index, index)
                };
            command.map(Some)
        }
        else {
            Ok(None)
        }
    }

    /// Parse an include command.
    fn include_command(&mut self, line: &str) -> Result<Command<T>> {
        let word = word(line);
        // NOTE: the line contains the word, hence unwrap.
        let index = line.find(word).unwrap();
        let after_index = index + word.len() + 1;
        self.column += after_index;
        try!(self.check_eol(line, after_index));
        Ok(Include(word.to_string()))
    }

    /// Parse a map command.
    fn map_command(&self, line: &str, mode: &str) -> Result<Command<T>> {
        let word = word(line);

        // NOTE: the line contains the word, hence unwrap.
        let index = line.find(word).unwrap();

        let rest = &line[index + word.len() ..].trim();
        if !rest.is_empty() {
            Ok(Map {
                action: rest.to_string(),
                keys: try!(parse_keys(word, self.line, self.column + index)),
                mode: mode.to_string(),
            })
        }
        else {
            Err(Box::new(Error::new(
                "<end of line>".to_string(),
                "mapping action".to_string(),
                Pos::new(self.line, self.column + line.len())
            )))
        }
    }

    /// Get an missing arguments error.
    fn missing_args(&self, column: usize) -> Box<Error> {
        Box::new(Error::new(
            "<end of line>".to_string(),
            "command arguments".to_string(),
            Pos::new(self.line, column)
        ))
    }

    /// Parse settings.
    pub fn parse<R: BufRead>(&mut self, input: R) -> Result<Vec<Command<T>>> {
        let mut commands = vec![];
        for (line_num, input_line) in input.lines().enumerate() {
            self.line = line_num + 1;
            if let Some(command) = try!(self.line(&try!(input_line))) {
                commands.push(command);
            }
        }
        Ok(commands)
    }

    /// Parse a single line of settings.
    pub fn parse_line(&mut self, line: &str) -> Result<Command<T>> {
        let mut commands = try!(self.parse(line.as_bytes()));
        match commands.pop() {
            Some(command) => Ok(command),
            None => Err(Box::new(Error::new(
                        "comment or <end of line>".to_string(),
                        "command".to_string(),
                        Pos::new(self.line, self.column + line.len())
                    )))
        }
    }

    /// Parse a set command.
    fn set_command(&mut self, line: &str) -> Result<Command<T>> {
        if let Some(words) = words(line, 2) {
            // NOTE: the line contains the word, hence unwrap.
            let index = line.find(words[0]).unwrap();
            let identifier = try!(check_ident(words[0].to_string(), &Pos::new(self.line, self.column + index)));

            let operator = words[1];
            // NOTE: the operator is in the line, hence unwrap.
            let operator_index = line.find(operator).unwrap();
            if operator == "=" {
                let rest = &line[operator_index + 1..];
                self.column += operator_index + 1;
                Ok(Set(identifier.to_string(), try!(self.value(rest))))
            }
            else {
                Err(Box::new(Error::new(
                    operator.to_string(),
                    "=".to_string(),
                    Pos::new(self.line, self.column + operator_index)
                )))
            }
        }
        else {
            Err(Box::new(Error::new(
                "<end of line>".to_string(),
                "=".to_string(),
                Pos::new(self.line, self.column + line.len()),
            )))
        }
    }

    /// Parse an unmap command.
    fn unmap_command(&mut self, line: &str, mode: &str) -> Result<Command<T>> {
        let word = word(line);

        // NOTE: the line contains the word, hence unwrap.
        let index = line.find(word).unwrap();

        let after_index = index + word.len() + 1;
        self.column += after_index;
        try!(self.check_eol(line, after_index));
        Ok(Unmap {
            keys: try!(parse_keys(word, self.line, self.column + index)),
            mode: mode.to_string(),
        })
    }

    /// Parse a value.
    fn value(&self, input: &str) -> Result<Value> {
        let string: String = input.chars().take_while(|&character| character != '#').collect();
        let string = string.trim();
        match string {
            "" => Err(Box::new(Error::new(
                      "<end of line>".to_string(),
                      "value".to_string(),
                      Pos::new(self.line, self.column + string.len())
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

/// Check if a string is an identifier.
fn check_ident(string: String, pos: &Pos) -> Result<String> {
    if string.chars().all(|character| character.is_alphanumeric() || character == '-' || character == '_') {
        if let Some(true) = string.chars().next().map(|character| character.is_alphabetic()) {
            return Ok(string)
        }
    }
    Err(Box::new(Error::new(string, "identifier".to_string(), pos.clone())))
}

/// Parse a single word.
fn maybe_word(input: &str) -> Option<&str> {
    input.split_whitespace()
        .next()
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
