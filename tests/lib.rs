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

extern crate mg_settings;
#[macro_use]
extern crate mg_settings_macros;

use mg_settings::{Config, EnumFromStr, Parser, ParseResult};
use mg_settings::Command::{self, App, Custom, Map, Set, Unmap};
use mg_settings::errors::Error;
use mg_settings::key::Key::{
    Alt,
    Backspace,
    Char,
    Control,
    Down,
    Enter,
    Escape,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Left,
    Right,
    Shift,
    Space,
    Tab,
    Up,
};
use mg_settings::Value::{Bool, Float, Int, Str};

use CustomCommand::*;

macro_rules! _assert_error {
    ($func:ident, $line:expr, $([$($error:expr),*]),*) => {
        let errors = $func($line);
        compare_errors!(errors, $([$($error),*]),*);
    };
}

macro_rules! assert_error {
    ($($tt:tt)*) => {
		_assert_error!(parse_error, $($tt)*);
    };
}

macro_rules! assert_error_config {
    ($($tt:tt)*) => {
		_assert_error!(parse_error_with_config, $($tt)*);
    };
}

macro_rules! compare_errors {
    ($errors:expr, $([$($error:expr),*]),*) => {
        let causes = [$(vec![$($error.to_string()),*]),*];
        assert_eq!($errors.len(), causes.len());
        for (error, causes) in $errors.iter().zip(causes.iter()) {
            let actual_causes: Vec<_> = error.iter()
                .map(ToString::to_string)
                .collect();
            assert_eq!(causes, &actual_causes);
        }
    };
}

#[derive(Commands, Debug, PartialEq)]
enum CustomCommand {
    Open(String),
    Quit,
    WinOpen(String),
}

type CommandParser = Parser<CustomCommand>;

#[test]
fn app_command() {
    assert_eq!(parse_string_with_config("complete-next"), vec![App("complete-next".to_string())]);
}

#[test]
fn commands_macro() {
    assert_eq!(Ok(Quit), CustomCommand::create("quit", ""));
    assert_eq!(Ok(Open("crates.io".to_string())), CustomCommand::create("open", "crates.io"));
    assert_eq!(Ok(WinOpen("crates.io".to_string())), CustomCommand::create("win-open", "crates.io"));
    assert_eq!(Ok(Quit), CustomCommand::create("quit", "crates.io"));
    assert_eq!(Err("unknown command ope".to_string()), CustomCommand::create("ope", ""));
}

#[test]
fn comments() {
    assert_eq!(parse_string("# Comment."), vec![]);
    assert_eq!(parse_string("set option1 = 5 # Comment."), vec![Set("option1".to_string(), Int(5))]);
}

#[test]
fn custom_commands() {
    assert_eq!(parse_string("quit"), vec![Custom(Quit)]);
    assert_eq!(parse_string("open crates.io"), vec![Custom(Open("crates.io".to_string()))]);
    assert_eq!(parse_string("win-open crates.io"), vec![Custom(WinOpen("crates.io".to_string()))]);
    assert_eq!(parse_string("open   crates.io  "), vec![Custom(Open("crates.io".to_string()))]);
    assert_eq!(parse_string("  open   crates.io  "), vec![Custom(Open("crates.io".to_string()))]);
}

#[test]
fn lexer_errors() {
    assert_error!("$ Comment.", ["unexpected $, expecting command or comment on line 1, column 1"]);
}

#[test]
fn newlines() {
    assert_error!("\n$ Comment.", ["unexpected $, expecting command or comment on line 2, column 1"]);
    assert_error!("\r\n$ Comment.", ["unexpected $, expecting command or comment on line 2, column 1"]);
    //assert_error!("\r$ Comment.", ["unexpected $, expecting command or comment on line 2, column 1"]);
}

#[test]
fn parser_errors() {
    assert_error!("set 5 5", ["unexpected 5, expecting identifier on line 1, column 5"]);
    assert_error!(" set 5 5", ["unexpected 5, expecting identifier on line 1, column 6"]);
    assert_error!("set  5 5", ["unexpected 5, expecting identifier on line 1, column 6"]);
    assert_error!("5", ["unexpected 5, expecting command or comment on line 1, column 1"]);
    assert_error!(" ste option1 = 42", ["unexpected ste, expecting command or comment on line 1, column 2"]);
    assert_error!("set option1 < 42", ["unexpected <, expecting = on line 1, column 13"]);
    assert_error!(" set option1 < 42", ["unexpected <, expecting = on line 1, column 14"]);
    assert_error!("set option1 =", ["unexpected <end of line>, expecting value on line 1, column 14"]);
    assert_error!("set", ["unexpected <end of line>, expecting command arguments on line 1, column 4"]);
    assert_error!("set option1", ["unexpected <end of line>, expecting = on line 1, column 12"]);
    assert_error!("include", ["unexpected <end of line>, expecting command arguments on line 1, column 8"]);
    assert_error_config!("nmap a", ["unexpected <end of line>, expecting mapping action on line 1, column 7"]);
    assert_error_config!("nmap", ["unexpected <end of line>, expecting command arguments on line 1, column 5"]);
    assert_error_config!("nmap <C-@> :open",
         ["failed to parse keys in map command",
        "unexpected @, expecting A-Z or special key on line 1, column 9"]);
    assert_error_config!("nmap <C-o@> :open",
        ["failed to parse keys in map command",
        "unexpected o@, expecting one character on line 1, column 9"]);
    assert_error_config!("nmap <C-TE> :open",
        ["failed to parse keys in map command",
        "unexpected TE, expecting one character on line 1, column 9"]);
    assert_error_config!("nmap <Test> :open",
        ["failed to parse keys in map command",
        "unexpected Test, expecting special key on line 1, column 7"]);
    assert_error_config!("mmap o :open", ["unexpected mmap, expecting command or comment on line 1, column 1"]);
    assert_error_config!("nunmap <F1> :help", ["unexpected :help, expecting <end of line> on line 1, column 13"]);
    assert_error_config!("include file.conf my-other-config", ["unexpected my-other-config, expecting <end of line> on line 1, column 19"]);
    assert_error_config!("include config my-other-config",
        ["unexpected my-other-config, expecting <end of line> on line 1, column 16"],
        ["failed to open included file `tests/config`",
        "No such file or directory (os error 2)"]);
    assert_error!("open", ["unexpected <end of line>, expecting command arguments on line 1, column 5"]);
    assert_error_config!("nmap <F1 :help",
        ["failed to parse keys in map command",
        "unexpected (none), expecting > on line 1, column 9"]);
    assert_error_config!("nmap <F> :help",
        ["failed to parse keys in map command",
        "unexpected F, expecting special key on line 1, column 7"]);
    assert_error_config!("nmap\nnmap <C-@> :open",
        ["unexpected <end of line>, expecting command arguments on line 1, column 5"],
        ["failed to parse keys in map command",
        "unexpected @, expecting A-Z or special key on line 2, column 9"]);
}

#[test]
fn include_command() {
    assert_eq!(parse_string("include file.conf"), vec![Set("option1".to_string(), Int(5))]);
    assert_eq!(parse_string("include  file.conf"), vec![Set("option1".to_string(), Int(5))]);
    assert_eq!(parse_string_no_include_path("include tests/file.conf"), vec![Set("option1".to_string(), Int(5))]);
}

#[test]
fn line() {
    let result = parse_line_with_config("nmap o :open");
    assert_eq!(result.commands,
        vec![Map { action: ":open".to_string(), keys: vec![Char('o')], mode: "n".to_string() }]);
    assert!(result.errors.is_empty());
    let result = parse_line_with_config("# nmap o :open");
    assert!(result.commands.is_empty());
    compare_errors!(result.errors,
        ["unexpected comment or <end of line>, expecting command on line 1, column 1"]);
}

#[test]
fn map_command() {
    assert_eq!(parse_string_with_config("nmap o :open"),
        vec![Map { action: ":open".to_string(), keys: vec![Char('o')], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <Backspace> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![Backspace], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <F1> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![F1], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <F2> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![F2], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <F3> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![F3], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <F4> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![F4], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <F5> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![F5], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <F6> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![F6], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <F7> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![F7], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <F8> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![F8], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <F9> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![F9], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <F10> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![F10], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <F11> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![F11], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <F12> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![F12], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <Down> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![Down], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <Enter> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![Enter], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <Esc> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![Escape], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <Left> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![Left], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap - :help"),
        vec![Map { action: ":help".to_string(), keys: vec![Char('-')], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap + :help"),
        vec![Map { action: ":help".to_string(), keys: vec![Char('+')], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <Right> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![Right], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <Space> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![Space], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <Tab> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![Tab], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <Up> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![Up], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <C-A> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![Control(Box::new(Char('A')))], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <C-Z> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![Control(Box::new(Char('Z')))], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <C-o> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![Control(Box::new(Char('o')))], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <A-o> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![Alt(Box::new(Char('o')))], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <S-A> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![Shift(Box::new(Char('A')))], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap Oo :open"),
        vec![Map { action: ":open".to_string(), keys: vec![Char('O'), Char('o')], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <C-O>o :open"),
        vec![Map { action: ":open".to_string(),
            keys: vec![Control(Box::new(Char('O'))), Char('o')], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <C-Tab> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![Control(Box::new(Tab))], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <S-Tab> :help"),
        vec![Map { action: ":help".to_string(), keys: vec![Shift(Box::new(Tab))], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <C-S-Tab> :help"),
        vec![Map { action: ":help".to_string(),
            keys: vec![Control(Box::new(Shift(Box::new(Tab))))], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <C-A-S-Tab> :help"),
        vec![Map { action: ":help".to_string(),
            keys: vec![Control(Box::new(Alt(Box::new(Shift(Box::new(Tab))))))], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <S-C-Tab> :help"),
        vec![Map { action: ":help".to_string(),
            keys: vec![Control(Box::new(Shift(Box::new(Tab))))], mode: "n".to_string() }]);

    let result = parse_with_config("nmap\nnmap <C-@> :open\nnmap o :open");
    assert_eq!(result.commands,
        vec![Map { action: ":open".to_string(), keys: vec![Char('o')], mode: "n".to_string() }]);
    compare_errors!(result.errors,
        ["unexpected <end of line>, expecting command arguments on line 1, column 5"],
        ["failed to parse keys in map command",
        "unexpected @, expecting A-Z or special key on line 2, column 9"]);
}

#[test]
fn set_command() {
    assert_eq!(parse_string("set option1 = 42"), vec![Set("option1".to_string(), Int(42))]);
    assert_eq!(parse_string("set option1 = 12.345"), vec![Set("option1".to_string(), Float(12.345))]);
    assert_eq!(parse_string("set option1 = false"), vec![Set("option1".to_string(), Bool(false))]);
    assert_eq!(parse_string("set option1 = true"), vec![Set("option1".to_string(), Bool(true))]);
    assert_eq!(parse_string("set option1 = value"), vec![Set("option1".to_string(), Str("value".to_string()))]);
    assert_eq!(parse_string("set option1 = value with spaces"), vec![Set("option1".to_string(), Str("value with spaces".to_string()))]);
    assert_eq!(parse_string("set option1 = 42\nset option2 = 12.345"), vec![Set("option1".to_string(), Int(42)), Set("option2".to_string(), Float(12.345))]);
    assert_eq!(parse_string("set option1 = 42\nset option2 = 12.345\n"), vec![Set("option1".to_string(), Int(42)), Set("option2".to_string(), Float(12.345))]);
    assert_eq!(parse_string("set option1 = 42\n\nset option2 = 12.345\n"), vec![Set("option1".to_string(), Int(42)), Set("option2".to_string(), Float(12.345))]);
    assert_eq!(parse_string("  set    option1    =    42    "), vec![Set("option1".to_string(), Int(42))]);
}

#[test]
fn unmap_command() {
    assert_eq!(parse_string_with_config("nunmap o"), vec![Unmap { keys: vec![Char('o')], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nunmap <F1>"), vec![Unmap { keys: vec![F1], mode: "n".to_string() }]);
}

fn parse_error(input: &str) -> Vec<Error> {
    let mut parser = CommandParser::new();
    parser.parse(input.as_bytes()).errors
}

fn parse_error_with_config(input: &str) -> Vec<Error> {
    let mut parser = CommandParser::new_with_config(Config {
        application_commands: vec![],
        mapping_modes: vec!["n", "i", "c"],
    });
    parser.set_include_path("tests");
    parser.parse(input.as_bytes(), ).errors
}

fn parse_string(input: &str) -> Vec<Command<CustomCommand>> {
    let mut parser = CommandParser::new();
    parser.set_include_path("tests");
    parser.parse(input.as_bytes()).commands
}

fn parse_string_no_include_path(input: &str) -> Vec<Command<CustomCommand>> {
    let mut parser = CommandParser::new();
    parser.parse(input.as_bytes()).commands
}

fn parse_line_with_config(input: &str) -> ParseResult<CustomCommand> {
    let mut parser = CommandParser::new_with_config(Config {
        application_commands: vec!["complete-next"],
        mapping_modes: vec!["n", "i", "c"],
    });
    parser.parse_line(input)
}

fn parse_with_config(input: &str) -> ParseResult<CustomCommand> {
    let mut parser = CommandParser::new_with_config(Config {
        application_commands: vec!["complete-next"],
        mapping_modes: vec!["n", "i", "c"],
    });
    parser.parse(input.as_bytes())
}

fn parse_string_with_config(input: &str) -> Vec<Command<CustomCommand>> {
    parse_with_config(input).commands
}
