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
    Delete,
    Down,
    End,
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
    Home,
    Insert,
    Left,
    PageDown,
    PageUp,
    Right,
    Shift,
    Space,
    Tab,
    Up,
};
use mg_settings::Value::{Bool, Float, Int, Str};

use CustomCommand::*;

macro_rules! _assert_error {
    ($func:ident, $line:expr, $($error:expr),*) => {
        let errors = $func($line);
        compare_errors!(errors, [$($error),*]);
    };
}

macro_rules! assert_custom_cmd {
    ($string_cmd:expr, $cmd:expr) => {
        assert_eq!(parse_string($string_cmd), vec![Custom($cmd)]);
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

macro_rules! assert_setting {
    ($key:expr, $value:expr, $setting:expr) => {
        assert_eq!(parse_string(&format!("set {} = {}", $key, $value)), vec![$setting]);
    };
}

macro_rules! assert_single_char {
    ($char:expr) => {
        assert_eq!(parse_string_with_config(&format!("nmap {} :open", $char)),
            vec![Map { action: ":open".to_string(), keys: vec![Char($char)], mode: "n".to_string() }]);
    };
}

macro_rules! assert_single_key {
    ($string_key:expr, $key:expr) => {
        assert_eq!(parse_string_with_config(&format!("nmap <{}> :help", $string_key)),
            vec![Map { action: ":help".to_string(), keys: vec![$key], mode: "n".to_string() }]);
    };
}

macro_rules! compare_errors {
    ($errors:expr, [$($error:expr),*]) => {
        let causes = [$($error.to_string()),*];
        assert_eq!($errors.len(), causes.len());
        for (error, causes) in $errors.iter().zip(causes.iter()) {
            assert_eq!(*causes, error.to_string());
        }
    };
}

#[derive(Commands, Debug, PartialEq)]
enum CustomCommand {
    Open(String),
    #[count]
    Scroll(Option<u32>),
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
    assert_eq!(Ok(Quit), CustomCommand::create("quit", "", None));
    assert_eq!(Ok(Open("crates.io".to_string())), CustomCommand::create("open", "crates.io", None));
    assert_eq!(Ok(WinOpen("crates.io".to_string())), CustomCommand::create("win-open", "crates.io", None));
    assert_eq!(Ok(Quit), CustomCommand::create("quit", "crates.io", None));
    assert_eq!(Err("unknown command ope".to_string()), CustomCommand::create("ope", "", None));
}

#[test]
fn comments() {
    assert_eq!(parse_string("# Comment."), vec![]);
    assert_eq!(parse_string("set option1 = 5 # Comment."), vec![Set("option1".to_string(), Int(5))]);
}

#[test]
fn custom_commands() {
    assert_custom_cmd!("quit", Quit);
    assert_custom_cmd!("open crates.io", Open("crates.io".to_string()));
    assert_custom_cmd!("win-open crates.io", WinOpen("crates.io".to_string()));
    assert_eq!(parse_string("open   crates.io  "), vec![Custom(Open("crates.io".to_string()))]);
    assert_eq!(parse_string("  open   crates.io  "), vec![Custom(Open("crates.io".to_string()))]);
}

#[test]
fn lexer_errors() {
    assert_error!("$ Comment.", "unexpected $, expecting command or comment on line 1, column 1");
}

#[test]
fn newlines() {
    assert_error!("\n$ Comment.", "unexpected $, expecting command or comment on line 2, column 1");
    assert_error!("\r\n$ Comment.", "unexpected $, expecting command or comment on line 2, column 1");
    //assert_error!("\r$ Comment.", ["unexpected $, expecting command or comment on line 2, column 1"]);
}

#[test]
fn parser_errors() {
    assert_error!("set 5 5", "unexpected 5, expecting identifier on line 1, column 5");
    assert_error!(" set 5 5", "unexpected 5, expecting identifier on line 1, column 6");
    assert_error!("set  5 5", "unexpected 5, expecting identifier on line 1, column 6");
    assert_error!("5", "unexpected 5, expecting command or comment on line 1, column 1");
    assert_error!(" ste option1 = 42", "unexpected ste, expecting command or comment on line 1, column 2");
    assert_error!("set option1 < 42", "unexpected <, expecting = on line 1, column 13");
    assert_error!(" set option1 < 42", "unexpected <, expecting = on line 1, column 14");
    assert_error!("set option1 =", "unexpected <end of line>, expecting value on line 1, column 14");
    assert_error!("set", "unexpected <end of line>, expecting command arguments on line 1, column 4");
    assert_error!("set option1", "unexpected <end of line>, expecting = on line 1, column 12");
    assert_error!("include", "unexpected <end of line>, expecting command arguments on line 1, column 8");
    assert_error_config!("nmap a", "unexpected <end of line>, expecting mapping action on line 1, column 7");
    assert_error_config!("nmap", "unexpected <end of line>, expecting command arguments on line 1, column 5");
    assert_error_config!("nmap <C-@> :open",
        "unexpected @, expecting A-Z or special key on line 1, column 9");
    assert_error_config!("nmap <C-o@> :open",
        "unexpected o@, expecting one character on line 1, column 9");
    assert_error_config!("nmap <C-TE> :open",
        "unexpected TE, expecting one character on line 1, column 9");
    assert_error_config!("nmap <Test> :open",
        "unexpected Test, expecting special key on line 1, column 7");
    assert_error_config!("mmap o :open", "unexpected mmap, expecting command or comment on line 1, column 1");
    assert_error_config!("nunmap <F1> :help", "unexpected :help, expecting <end of line> on line 1, column 13");
    assert_error_config!("include file.conf my-other-config", "unexpected my-other-config, expecting <end of line> on line 1, column 19");
    assert_error_config!("include config my-other-config",
        "unexpected my-other-config, expecting <end of line> on line 1, column 16",
        "failed to open included file `tests/config`: No such file or directory (os error 2)");
    assert_error!("open", "unexpected <end of line>, expecting command arguments on line 1, column 5");
    assert_error_config!("nmap <F1 :help",
        "unexpected (none), expecting > on line 1, column 9");
    assert_error_config!("nmap <F> :help",
        "unexpected F, expecting special key on line 1, column 7");
    assert_error_config!("nmap\nnmap <C-@> :open",
        "unexpected <end of line>, expecting command arguments on line 1, column 5",
        "unexpected @, expecting A-Z or special key on line 2, column 9");
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
    compare_errors!(result.errors, ["unexpected comment or <end of line>, expecting command on line 1, column 1"]);
}

#[test]
fn map_command() {
    assert_single_key!("Backspace", Backspace);
    assert_single_key!("Delete", Delete);
    assert_single_key!("Down", Down);
    assert_single_key!("End", End);
    assert_single_key!("Enter", Enter);
    assert_single_key!("Esc", Escape);
    assert_single_key!("F1", F1);
    assert_single_key!("F2", F2);
    assert_single_key!("F3", F3);
    assert_single_key!("F4", F4);
    assert_single_key!("F5", F5);
    assert_single_key!("F6", F6);
    assert_single_key!("F7", F7);
    assert_single_key!("F8", F8);
    assert_single_key!("F9", F9);
    assert_single_key!("F10", F10);
    assert_single_key!("F11", F11);
    assert_single_key!("F12", F12);
    assert_single_key!("Home", Home);
    assert_single_key!("Insert", Insert);
    assert_single_key!("Left", Left);
    assert_single_key!("PageDown", PageDown);
    assert_single_key!("PageUp", PageUp);
    assert_single_key!("Right", Right);
    assert_single_key!("Space", Space);
    assert_single_key!("Tab", Tab);
    assert_single_key!("Up", Up);
    assert_single_key!("C-A", Control(Box::new(Char('A'))));
    assert_single_key!("C-Z", Control(Box::new(Char('Z'))));
    assert_single_key!("C-o", Control(Box::new(Char('o'))));
    assert_single_key!("A-o", Alt(Box::new(Char('o'))));
    assert_single_key!("S-A", Shift(Box::new(Char('A'))));
    assert_single_key!("C-Tab", Control(Box::new(Tab)));
    assert_single_key!("S-Tab", Shift(Box::new(Tab)));
    assert_single_key!("C-S-Tab", Control(Box::new(Shift(Box::new(Tab)))));
    assert_single_key!("C-A-S-Tab", Control(Box::new(Alt(Box::new(Shift(Box::new(Tab)))))));
    assert_single_key!("S-C-Tab", Control(Box::new(Shift(Box::new(Tab)))));

    assert_single_char!('o');
    assert_single_char!('-');
    assert_single_char!('+');

    assert_eq!(parse_string_with_config("nmap Oo :open"),
        vec![Map { action: ":open".to_string(), keys: vec![Char('O'), Char('o')], mode: "n".to_string() }]);
    assert_eq!(parse_string_with_config("nmap <C-O>o :open"),
        vec![Map { action: ":open".to_string(),
            keys: vec![Control(Box::new(Char('O'))), Char('o')], mode: "n".to_string() }]);

    let result = parse_with_config("nmap\nnmap <C-@> :open\nnmap o :open");
    assert_eq!(result.commands,
        vec![Map { action: ":open".to_string(), keys: vec![Char('o')], mode: "n".to_string() }]);
    compare_errors!(result.errors,
        ["unexpected <end of line>, expecting command arguments on line 1, column 5",
        "unexpected @, expecting A-Z or special key on line 2, column 9"]);
}

#[test]
fn set_command() {
    assert_setting!("option1", "42", Set("option1".to_string(), Int(42)));
    assert_setting!("option1", "12.345", Set("option1".to_string(), Float(12.345)));
    assert_setting!("option1", "false", Set("option1".to_string(), Bool(false)));
    assert_setting!("option1", "true", Set("option1".to_string(), Bool(true)));
    assert_setting!("option1", "value", Set("option1".to_string(), Str("value".to_string())));
    assert_setting!("option1", "value with spaces", Set("option1".to_string(), Str("value with spaces".to_string())));
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
    parser.parse(input.as_bytes(), None).errors
}

fn parse_error_with_config(input: &str) -> Vec<Error> {
    let mut parser = CommandParser::new_with_config(Config {
        application_commands: vec![],
        mapping_modes: vec!["n", "i", "c"],
    });
    parser.set_include_path("tests");
    parser.parse(input.as_bytes(), None).errors
}

fn parse_string(input: &str) -> Vec<Command<CustomCommand>> {
    let mut parser = CommandParser::new();
    parser.set_include_path("tests");
    parser.parse(input.as_bytes(), None).commands
}

fn parse_string_no_include_path(input: &str) -> Vec<Command<CustomCommand>> {
    let mut parser = CommandParser::new();
    parser.parse(input.as_bytes(), None).commands
}

fn parse_line_with_config(input: &str) -> ParseResult<CustomCommand> {
    let mut parser = CommandParser::new_with_config(Config {
        application_commands: vec!["complete-next"],
        mapping_modes: vec!["n", "i", "c"],
    });
    parser.parse_line(input, None)
}

fn parse_with_config(input: &str) -> ParseResult<CustomCommand> {
    let mut parser = CommandParser::new_with_config(Config {
        application_commands: vec!["complete-next"],
        mapping_modes: vec!["n", "i", "c"],
    });
    parser.parse(input.as_bytes(), None)
}

fn parse_string_with_config(input: &str) -> Vec<Command<CustomCommand>> {
    parse_with_config(input).commands
}
