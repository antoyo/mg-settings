/*
 * Copyright (c) 2016-2017 Boucher, Antoni <bouanto@zoho.com>
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
 * TODO: prevent mutual includes.
 * TODO: auto-include files.
 * TODO: support set = without spaces around =.
 * TODO: Add array type.
 */

pub mod errors;
mod file;
pub mod key;
#[doc(hidden)]
pub mod position;
pub mod settings;
mod string;

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use errors::{Error, ParseError, Result};
use errors::ErrorType::{MissingArgument, NoCommand, Parse, UnknownCommand};
use key::{Key, parse_keys};
use position::Pos;
use string::{StrExt, check_ident, maybe_word, word, words};

use Command::*;
use Value::*;

#[macro_export]
macro_rules! rtry {
    ($parse_result:expr, $result:expr) => {
        rtry_no_return!($parse_result, $result, { return $parse_result; });
    };
}

#[macro_export]
macro_rules! rtry_no_return {
    ($parse_result:expr, $result:expr, $error_block:block) => {
        match $result {
            Ok(result) => result,
            Err(error) => {
                $parse_result.errors.push(error.into());
                $error_block
            },
        }
    };
}

/// Trait to specify the completion values for a type.
pub trait CompletionValues {
    /// Get the completion values for the type.
    fn completion_values() -> Vec<String>;
}

impl CompletionValues for bool {
    fn completion_values() -> Vec<String> {
        vec!["true".to_string(), "false".to_string()]
    }
}

impl CompletionValues for i64 {
    fn completion_values() -> Vec<String> {
        vec![]
    }
}

impl CompletionValues for String {
    fn completion_values() -> Vec<String> {
        vec![]
    }
}

/// The `EnumFromStr` trait is used to specify how to construct an enum value from a string.
pub trait EnumFromStr
    where Self: Sized
{
    /// Create the enum value from the `variant` string and an `argument` string.
    fn create(variant: &str, argument: &str, prefix: Option<u32>) -> std::result::Result<Self, String>;

    /// Check wether the enum variant has an argument.
    fn has_argument(variant: &str) -> std::result::Result<bool, String>;
}

/// Tre `EnumMetaData` trait is used to get associated meta-data for the enum variants.
/// The meta-data is specified using the following attributes:
/// ``` ignore
/// #[completion(hidden)]
/// #[special_command]
/// #[help(Command help)]
/// ```
pub trait EnumMetaData {
    /// Get the metadata associated with the enum.
    fn get_metadata() -> HashMap<String, MetaData>;
}

/// Command/setting meta-data coming from the attributes.
/// See `EnumMetaData` to see the list of supported attributes.
#[derive(Debug)]
pub struct MetaData {
    /// Whether this command/setting should be shown in the completion or not.
    pub completion_hidden: bool,
    /// The help text associated with this command/setting.
    pub help_text: String,
    /// Whether this is a special command or not.
    /// This is not applicable to settings.
    pub is_special_command: bool,
}

/// The commands and errors from parsing a config file.
pub struct ParseResult<T> {
    /// The parsed commands.
    pub commands: Vec<Command<T>>,
    /// The errors resulting from the parsing.
    pub errors: Vec<Error>,
}

impl<T> ParseResult<T> {
    #[allow(unknown_lints, new_without_default_derive)]
    /// Create a new empty parser result.
    pub fn new() -> Self {
        ParseResult {
            commands: vec![],
            errors: vec![],
        }
    }

    fn new_with_command(command: Command<T>) -> Self {
        ParseResult {
            commands: vec![command],
            errors: vec![],
        }
    }

    fn merge(&mut self, mut parse_result: ParseResult<T>) {
        self.commands.append(&mut parse_result.commands);
        self.errors.append(&mut parse_result.errors);
    }
}

/// Trait specifying the value completions for settings.
pub trait SettingCompletion {
    /// Get the value completions of all the setting.
    fn get_value_completions() -> HashMap<String, Vec<String>>;
}

/// The `Command` enum represents a command from a config file.
#[derive(Debug, PartialEq)]
pub enum Command<T> {
    /// A command from the application library.
    App(String),
    /// A custom command.
    Custom(T),
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
    /// The application library commands.
    pub application_commands: Vec<&'static str>,
    /// The available mapping modes for the map command.
    pub mapping_modes: Vec<&'static str>,
}

/// The config parser.
pub struct Parser<T> {
    column: usize,
    config: Config,
    include_path: PathBuf,
    line: usize,
    _phantom: PhantomData<T>,
}

impl<T: EnumFromStr> Parser<T> {
    #[allow(unknown_lints, new_without_default_derive)]
    /// Create a new parser without config.
    pub fn new() -> Self {
        Parser {
            column: 1,
            config: Config::default(),
            include_path: Path::new("./").to_path_buf(),
            line: 1,
            _phantom: PhantomData,
        }
    }

    /// Create a new parser with config.
    pub fn new_with_config(config: Config) -> Self {
        Parser {
            column: 1,
            config: config,
            include_path: Path::new("./").to_path_buf(),
            line: 1,
            _phantom: PhantomData,
        }
    }

    /// Check that we reached the end of the line.
    fn check_eol(&self, line: &str, index: usize) -> Result<()> {
        if line.len() > index {
            let rest = &line[index..];
            if let Some(word) = maybe_word(rest) {
                let index = word.index;
                return Err(ParseError::new(
                    Parse,
                    rest.to_string(),
                    "<end of line>".to_string(),
                    Pos::new(self.line, self.column + index),
                ));
            }
        }
        Ok(())
    }

    /// Parse a custom command or return an error if it does not exist.
    fn custom_command(&self, line: &str, word: &str, start_index: usize, index: usize, prefix: Option<u32>)
        -> Result<Command<T>>
    {
        let args =
            if line.len() > start_index {
                line[start_index..].trim()
            }
            else if let Ok(true) = T::has_argument(word) {
                return Err(self.missing_args(start_index));
            }
            else {
                ""
            };
        if let Ok(command) = T::create(word, args, prefix) {
            Ok(Custom(command))
        }
        else if self.config.application_commands.contains(&word) {
            Ok(App(word.to_string()))
        }
        else {
            return Err(ParseError::new(
                UnknownCommand,
                word.to_string(),
                "command or comment".to_string(),
                Pos::new(self.line, index + 1)
            ))
        }
    }

    /// Get the rest of the line, starting at `column`, returning an error if the column is greater
    /// than the line's length.
    fn get_rest<'a>(&self, line: &'a str, column: usize) -> Result<&'a str> {
        if line.len() > column {
            Ok(&line[column..])
        }
        else {
            Err(self.missing_args(column))
        }
    }

    /// Parse a line.
    fn line(&mut self, line: &str, prefix: Option<u32>) -> ParseResult<T> {
        let mut result = ParseResult::new();
        if let Some(word) = maybe_word(line) {
            let index = word.index;
            let word = word.word;
            let start_index = index + word.len() + 1;
            self.column = start_index + 1;

            let (start3, end3) = word.rsplit_at(3);
            let (start5, end5) = word.rsplit_at(5);
            if word.starts_with('#') {
                return result;
            }

            if word == "include" {
                let rest = rtry!(result, self.get_rest(line, start_index));
                self.include_command(rest)
            }
            else {
                let command =
                    if word == "set" {
                        let rest = rtry!(result, self.get_rest(line, start_index));
                        self.set_command(rest)
                    }
                    else if end3 == "map" && self.config.mapping_modes.contains(&start3) {
                        let rest = rtry!(result, self.get_rest(line, start_index));
                        self.map_command(rest, start3)
                    }
                    else if end5 == "unmap" && self.config.mapping_modes.contains(&start5) {
                        let rest = rtry!(result, self.get_rest(line, start_index));
                        self.unmap_command(rest, start5)
                    }
                    else {
                        self.custom_command(line, word, start_index, index, prefix)
                    };
                let command = rtry!(result, command);
                ParseResult::new_with_command(command)
            }
        }
        else {
            result
        }
    }

    /// Parse an include command.
    fn include_command(&mut self, line: &str) -> ParseResult<T> {
        let word = word(line);
        let index = word.index;
        let word = word.word;
        let after_index = index + word.len() + 1;
        self.column += after_index;
        let mut result = ParseResult::new();
        rtry_no_return!(result, self.check_eol(line, after_index), {});
        let path = Path::new(&self.include_path).join(word);
        let file = rtry!(result, file::open(&path));
        let buf_reader = BufReader::new(file);
        result.merge(self.parse(buf_reader, None));
        result
    }

    /// Parse a map command.
    fn map_command(&self, line: &str, mode: &str) -> Result<Command<T>> {
        let word = word(line);
        let index = word.index;
        let word = word.word;
        let rest = &line[index + word.len() ..].trim();
        if !rest.is_empty() {
            Ok(Map {
                action: rest.to_string(),
                keys: parse_keys(word, self.line, self.column + index)?,
                mode: mode.to_string(),
            })
        }
        else {
            Err(ParseError::new(
                Parse,
                "<end of line>".to_string(),
                "mapping action".to_string(),
                Pos::new(self.line, self.column + line.len())
            ))
        }
    }

    /// Get an missing arguments error.
    fn missing_args(&self, column: usize) -> Error {
        ParseError::new(
            MissingArgument,
            "<end of line>".to_string(),
            "command arguments".to_string(),
            Pos::new(self.line, column)
        )
    }

    /// Parse settings.
    pub fn parse<R: BufRead>(&mut self, input: R, prefix: Option<u32>) -> ParseResult<T> {
        let mut result = ParseResult::new();
        for (line_num, input_line) in input.lines().enumerate() {
            self.line = line_num + 1;
            let input_line = rtry_no_return!(result, input_line, { continue });
            result.merge(self.line(&input_line, prefix));
        }
        result
    }

    /// Parse a single line of settings.
    pub fn parse_line(&mut self, line: &str, prefix: Option<u32>) -> ParseResult<T> {
        let mut result = self.parse(line.as_bytes(), prefix);
        if result.commands.is_empty() && result.errors.is_empty() {
            result.errors.push(ParseError::new(
                NoCommand,
                "comment or <end of line>".to_string(),
                "command".to_string(),
                Pos::new(self.line, 1)
            ));
        }
        result
    }

    /// Parse a set command.
    fn set_command(&mut self, line: &str) -> Result<Command<T>> {
        if let Some(words) = words(line, 2) {
            let index = words[0].index;
            let word =  words[0].word;
            let identifier = check_ident(word.to_string(), &Pos::new(self.line, self.column + index))?;

            let operator = words[1].word;
            let operator_index = words[1].index;
            if operator == "=" {
                let rest = &line[operator_index + 1..];
                self.column += operator_index + 1;
                Ok(Set(identifier.to_string(), self.value(rest)?))
            }
            else {
                return Err(ParseError::new(
                    Parse,
                    operator.to_string(),
                    "=".to_string(),
                    Pos::new(self.line, self.column + operator_index)
                ))
            }
        }
        else {
            return Err(ParseError::new(
                Parse,
                "<end of line>".to_string(),
                "=".to_string(),
                Pos::new(self.line, self.column + line.len()),
            ))
        }
    }

    /// Set the directory where the include command will look for files to include.
    pub fn set_include_path<P: AsRef<Path>>(&mut self, directory: P) {
        self.include_path = directory.as_ref().to_path_buf();
    }

    /// Parse an unmap command.
    fn unmap_command(&mut self, line: &str, mode: &str) -> Result<Command<T>> {
        let word = word(line);
        let index = word.index;
        let word = word.word;
        let after_index = index + word.len() + 1;
        self.column += after_index;
        self.check_eol(line, after_index)?;
        Ok(Unmap {
            keys: parse_keys(word, self.line, self.column + index)?,
            mode: mode.to_string(),
        })
    }

    /// Parse a value.
    fn value(&self, input: &str) -> Result<Value> {
        let string: String = input.chars().take_while(|&character| character != '#').collect();
        let string = string.trim();
        match string {
            "" => Err(ParseError::new(
                      Parse,
                      "<end of line>".to_string(),
                      "value".to_string(),
                      Pos::new(self.line, self.column + string.len())
                  )),
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

/// Trait for converting an identifier like "/" to a special command.
pub trait SpecialCommand
    where Self: Sized
{
    /// Convert an identifier like "/" to a special command.
    fn identifier_to_command(identifier: char, input: &str) -> std::result::Result<Self, String>;

    /// Check if a character is a special command identifier.
    fn is_identifier(character: char) -> bool;

    /// Check if the identifier is declared `always`.
    /// The always option means that the command is activated every time a character is typed (like
    /// incremental search).
    fn is_incremental(identifier: char) -> bool;
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

impl Value {
    /// Get a string representation of the value.
    pub fn to_type(&self) -> &str {
        match *self {
            Bool(_) => "bool",
            Float(_) => "float",
            Int(_) => "int",
            Str(_) => "string",
        }
    }
}
