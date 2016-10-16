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

//! Type for representing keys and functions for parsing strings into `Key`s.

use error::{Error, Result};
use error::ErrorType::Parse;
use position::Pos;

use self::Key::*;

/// Enum representing the keys that can be used in a mapping.
#[derive(Debug, Eq, Hash, PartialEq)]
pub enum Key {
    /// A single-character key.
    Char(char),
    /// Control + another key.
    Control(Box<Key>),
    /// Down arrow.
    Down,
    /// Enter key.
    Enter,
    /// Escape key.
    Escape,
    /// Function key 1
    F1,
    /// Function key 2
    F2,
    /// Function key 3
    F3,
    /// Function key 4
    F4,
    /// Function key 5
    F5,
    /// Function key 6
    F6,
    /// Function key 7
    F7,
    /// Function key 8
    F8,
    /// Function key 9
    F9,
    /// Function key 10
    F10,
    /// Function key 11
    F11,
    /// Function key 12
    F12,
    /// Left arrow.
    Left,
    /// Minus.
    Minus,
    /// Plus.
    Plus,
    /// Right arrow.
    Right,
    /// Space key.
    Space,
    /// Tab key.
    Tab,
    /// Up arrow.
    Up,
}

fn parse_key(input: &str, line_num: usize, column_num: usize) -> Result<(Key, usize)> {
    let mut chars = input.chars();
    let result =
        match chars.next() {
            Some('<') => {
                let key: String = chars.take_while(|&character| character != '>').collect();
                let (start, end) = key.split_at(2);
                if start == "C-" && end.len() == 1 {
                    let character = end.chars().next().unwrap(); // NOTE: There is one character, hence unwrap.
                    match character {
                        'A' ... 'Z' | 'a' ... 'z' => (Control(Box::new(Char(character))), 5),
                        _ => return Err(Box::new(Error::new(
                                 Parse,
                                 character.to_string(),
                                 "A-Z".to_string(),
                                 Pos::new(line_num, column_num + 3)
                             ))),
                    }
                }
                else {
                    match key.as_str() {
                        "Down" => (Down, 6),
                        "Enter" => (Enter, 7),
                        "Esc" => (Escape, 5),
                        "F1" => (F1, 4),
                        "F2" => (F2, 4),
                        "F3" => (F3, 4),
                        "F4" => (F4, 4),
                        "F5" => (F5, 4),
                        "F6" => (F6, 4),
                        "F7" => (F7, 4),
                        "F8" => (F8, 4),
                        "F9" => (F9, 4),
                        "F10" => (F10, 5),
                        "F11" => (F11, 5),
                        "F12" => (F12, 5),
                        "Left" => (Left, 6),
                        "Minus" => (Minus, 7),
                        "Plus" => (Plus, 6),
                        "Right" => (Right, 7),
                        "Space" => (Space, 7),
                        "Tab" => (Tab, 5),
                        "Up" => (Up, 4),
                        _ => return Err(Box::new(Error::new(
                                 Parse,
                                 key.clone(),
                                 "special key".to_string(),
                                 Pos::new(line_num, column_num + 1)
                             ))),
                    }
                }
            },
            Some(character @ 'A' ... 'Z') | Some(character @ 'a' ... 'z') => (Char(character), 1),
            Some(character) => return Err(Box::new(Error::new(
                Parse,
                character.to_string(),
                "key".to_string(),
                Pos::new(line_num, column_num)
            ))),
            None => unreachable!() ,
        };
    Ok(result)
}

/// Parse a string into a vector of `Key`s.
pub fn parse_keys(mut input: &str, line_num: usize, column_num: usize) -> Result<Vec<Key>> {
    let mut keys = vec![];
    let mut index = 0;
    while !input.is_empty() {
        let (key, size) = try!(parse_key(input, line_num, column_num + index));
        keys.push(key);
        input = &input[size..];
        index += size;
    }
    Ok(keys)
}
