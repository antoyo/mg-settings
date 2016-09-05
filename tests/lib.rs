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

extern crate mg_settings;

use mg_settings::parse;
use mg_settings::Command::{self, Set};
use mg_settings::Value::{Bool, Float, Int, Str};

#[test]
fn comments() {
    assert_eq!(parse_string("# Comment."), vec![]);
    assert_eq!(parse_string("set option1 = 5 # Comment."), vec![Set("option1".to_string(), Int(5))]);
}

#[test]
fn lexer_errors() {
    assert_eq!(parse_error("$ Comment."), "unexpected $, expecting command or comment on line 1, column 1".to_string());
}

#[test]
fn newlines() {
    assert_eq!(parse_error("\n$ Comment."), "unexpected $, expecting command or comment on line 2, column 1".to_string());
    assert_eq!(parse_error("\r\n$ Comment."), "unexpected $, expecting command or comment on line 2, column 1".to_string());
    //assert_eq!(parse_error("\r$ Comment."), "unexpected $, expecting command or comment on line 2, column 1".to_string());
}

#[test]
fn parser_errors() {
    assert_eq!(parse_error("set 5 5"), "unexpected 5, expecting identifier on line 1, column 5".to_string());
    assert_eq!(parse_error("set  5 5"), "unexpected 5, expecting identifier on line 1, column 6".to_string());
    assert_eq!(parse_error("5"), "unexpected 5, expecting command or comment on line 1, column 1".to_string());
    assert_eq!(parse_error(" ste option1 = 42"), "unexpected ste, expecting command or comment on line 1, column 2".to_string());
    assert_eq!(parse_error("set option1 < 42"), "unexpected <, expecting = on line 1, column 13".to_string());
    assert_eq!(parse_error(" set option1 < 42"), "unexpected <, expecting = on line 1, column 14".to_string());
}

#[test]
fn set_command() {
    assert_eq!(parse_string("set option1 = 42"), vec![Set("option1".to_string(), Int(42))]);
    assert_eq!(parse_string("set option1 = 3.141592"), vec![Set("option1".to_string(), Float(3.141592))]);
    assert_eq!(parse_string("set option1 = false"), vec![Set("option1".to_string(), Bool(false))]);
    assert_eq!(parse_string("set option1 = true"), vec![Set("option1".to_string(), Bool(true))]);
    assert_eq!(parse_string("set option1 = value"), vec![Set("option1".to_string(), Str("value".to_string()))]);
    assert_eq!(parse_string("set option1 = value with spaces"), vec![Set("option1".to_string(), Str("value with spaces".to_string()))]);
    assert_eq!(parse_string("set option1 = 42\nset option2 = 3.141592"), vec![Set("option1".to_string(), Int(42)), Set("option2".to_string(), Float(3.141592))]);
    assert_eq!(parse_string("set option1 = 42\nset option2 = 3.141592\n"), vec![Set("option1".to_string(), Int(42)), Set("option2".to_string(), Float(3.141592))]);
    assert_eq!(parse_string("set option1 = 42\n\nset option2 = 3.141592\n"), vec![Set("option1".to_string(), Int(42)), Set("option2".to_string(), Float(3.141592))]);
}

fn parse_error(input: &str) -> String {
    parse(input.as_bytes()).unwrap_err().to_string()
}

fn parse_string(input: &str) -> Vec<Command> {
    parse(input.as_bytes()).unwrap()
}
