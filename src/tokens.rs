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

use std::fmt::{self, Display, Formatter};
use std::iter::Peekable;

use error::Error;
use position::{Pos, WithPos};

use self::Token::*;

pub struct InputTokens<'a> {
    input: &'a str,
    pos: Pos,
}

impl<'a> InputTokens<'a> {
    /// Advance the position and the input.
    fn advance(&mut self, len: usize) {
        let mut chars = self.input.chars();
        match chars.next() {
            Some('\r') => {
                if let Some('\n') = chars.next() {
                    self.input = &self.input[len..];
                }
                debug_assert_eq!(len, 1);
                self.pos.newline();
            },
            Some('\n') => {
                debug_assert_eq!(len, 1);
                self.pos.newline();
            },
            Some(_) => self.pos.column += len as u32,
            None => (),
        }
        self.input = &self.input[len..];
    }

    fn int_or_float(&mut self) -> WithPos<Token> {
        let pos = self.pos.clone();
        let num: String = self.input.chars()
            .take_while(|&character| character.is_numeric() || character == '.')
            .collect();
        self.advance(num.len());
        let token =
            if num.contains('.') {
                Float(num.parse().unwrap()) // NOTE: The string only contains digits or dot, hence unwrap.
            }
            else {
                Int(num.parse().unwrap()) // NOTE: The string only contains digits, hence unwrap.
            };
        WithPos::new(token, pos)
    }

    fn keyword_or_string(&mut self) -> WithPos<Token> {
        let pos = self.pos.clone();
        let mut skip = false;
        let mut backslash_count = 0;
        let mut string = String::new();
        for character in self.input.chars() {
            match character {
                '\\' => {
                    skip = true;
                    backslash_count += 1;
                },
                ' ' if skip => skip = false,
                _ if !character.is_alphanumeric() => break,
                _ => skip = false,
            }
            if !skip {
                string.push(character);
            }
        }
        self.advance(string.len() + backslash_count);
        let token =
            match string.as_str() {
                "false" => Bool(false),
                "set" => Set,
                "true" => Bool(true),
                _ => Str(string),
            };
        WithPos::new(token, pos)
    }

    fn skip_comment(&mut self) {
        let index = self.input.chars()
            .position(|character| character == '\n');
        if let Some(index) = index {
            self.advance(index);
        }
        else {
            self.advance(self.input.len());
        }
    }

    fn quoted_string(&mut self) -> Result<WithPos<Token>> {
        let mut pos = self.pos.clone();
        self.advance(1);
        let string: String = self.input.chars()
            .take_while(|&character| character != '"' && character != '\r' && character != '\n')
            .collect();
        self.advance(string.len());
        match self.input.chars().next() {
            Some('"') => {
                self.advance(1);
                Ok(WithPos::new(QuotedStr(string), pos))
            },
            Some(character) => {
                pos.column += string.len() as u32;
                Err(Error::new(character.to_string(), "\"".into(), pos))
            },
            None => {
                pos.column += string.len() as u32;
                Err(Error::new("eof".into(), "\"".into(), pos))
            },
        }
    }
}

impl<'a> Iterator for InputTokens<'a> {
    type Item = Result<WithPos<Token>>;

    fn next(&mut self) -> Option<Self::Item> {
        let item =
            match self.input.chars().next() {
                Some('#') => {
                    self.skip_comment();
                    return self.next();
                },
                Some('"') => self.quoted_string(),
                Some(character) if character.is_alphabetic() =>
                    Ok(self.keyword_or_string()),
                Some(character) if character.is_numeric() =>
                    Ok(self.int_or_float()),
                Some(character) if character.is_whitespace() => {
                    self.advance(1);
                    return self.next()
                },
                Some(character) => {
                    Err(Error::new(
                        character.to_string(),
                        "identifier, number, boolean, string or comment".into(),
                        self.pos.clone()
                    ))
                },
                None => Ok(WithPos::new(Eof, self.pos.clone())),
            };
        Some(item)
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Token {
    Bool(bool),
    Eof,
    Float(f64),
    Int(i64),
    Set,
    Str(String),
    QuotedStr(String),
}

impl Display for Token {
    fn fmt(&self, formatter: &mut Formatter) -> ::std::result::Result<(), fmt::Error> {
        let string =
            match *self {
                Bool(bool) => bool.to_string(),
                Eof => "eof".to_string(),
                Float(float) => float.to_string(),
                Int(int) => int.to_string(),
                Set => "set".to_string(),
                Str(ref string) | QuotedStr(ref string) => string.clone(),
            };
        write!(formatter, "{}", string)
    }
}

pub type Tokens<'a> = Peekable<InputTokens<'a>>;

/// Create an iterator of tokens from the input string.
pub fn tokenize(input: &str) -> Tokens {
    InputTokens {
        input: input,
        pos: Pos::new(1, 1),
    }.peekable()
}
