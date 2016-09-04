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

use mg_settings::error::Error;
use mg_settings::parse;
use mg_settings::Command::Set;
use mg_settings::position::Pos;
use mg_settings::Value::{Bool, Float, Int, Str};

#[test]
fn comments() {
    assert_eq!(parse("# Comment."), Ok(vec![]));
    assert_eq!(parse("set option1 5 # Comment."), Ok(vec![Set("option1".into(), Int(5))]));
}

#[test]
fn lexer_errors() {
    assert_eq!(parse("$ Comment."), Err(Error::new("$".into(), "identifier, number, boolean, string or comment".into(), Pos::new(1, 1))));
}

#[test]
fn newlines() {
    assert_eq!(parse("\n$ Comment."), Err(Error::new("$".into(), "identifier, number, boolean, string or comment".into(), Pos::new(2, 1))));
    assert_eq!(parse("\r\n$ Comment."), Err(Error::new("$".into(), "identifier, number, boolean, string or comment".into(), Pos::new(2, 1))));
    assert_eq!(parse("\r$ Comment."), Err(Error::new("$".into(), "identifier, number, boolean, string or comment".into(), Pos::new(2, 1))));
}

#[test]
fn parser_errors() {
    assert_eq!(parse("set 5 5"), Err(Error::new("5".into(), "identifier".into(), Pos::new(1, 5))));
    assert_eq!(parse("set option set"), Err(Error::new("set".into(), "value".into(), Pos::new(1, 12))));
    assert_eq!(parse("5"), Err(Error::new("5".into(), "command or comment".into(), Pos::new(1, 1))));
    assert_eq!(parse("set true 5"), Err(Error::new("true".into(), "identifier".into(), Pos::new(1, 5))));
    assert_eq!(parse("set option\\ with\\ spaces 5"), Err(Error::new("option with spaces".into(), "identifier".into(), Pos::new(1, 5))));
    assert_eq!(parse("set option1 \"value with spaces"), Err(Error::new("eof".into(), "\"".into(), Pos::new(1, 30))));
    assert_eq!(parse("set option1 \"value with spaces\n"), Err(Error::new("\n".into(), "\"".into(), Pos::new(1, 30))));
    assert_eq!(parse("set \"option\" 5"), Err(Error::new("option".into(), "identifier".into(), Pos::new(1, 5))));
}

#[test]
fn set_command() {
    assert_eq!(parse("set option1 42"), Ok(vec![Set("option1".into(), Int(42))]));
    assert_eq!(parse("set option1 3.141592"), Ok(vec![Set("option1".into(), Float(3.141592))]));
    assert_eq!(parse("set option1 false"), Ok(vec![Set("option1".into(), Bool(false))]));
    assert_eq!(parse("set option1 true"), Ok(vec![Set("option1".into(), Bool(true))]));
    assert_eq!(parse("set option1 value"), Ok(vec![Set("option1".into(), Str("value".into()))]));
    assert_eq!(parse("set option1 value\\ with\\ spaces"), Ok(vec![Set("option1".into(), Str("value with spaces".into()))]));
    assert_eq!(parse("set option1 \"value with spaces\""), Ok(vec![Set("option1".into(), Str("value with spaces".into()))]));
    assert_eq!(parse("set option1 42\nset option2 3.141592"), Ok(vec![Set("option1".into(), Int(42)), Set("option2".into(), Float(3.141592))]));
}
