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

//! Type for representing keys and functions for parsing strings into `Key`s.

use std::fmt::{self, Display, Formatter};

use errors::{ParseError, Result};
use errors::ErrorType::Parse;
use position::Pos;

use self::Key::*;

/// Structure to represent which keys were pressed.
struct ConstructorKeys {
    alt: bool,
    control: bool,
    shift: bool,
}

impl ConstructorKeys {
    fn new() -> Self {
        ConstructorKeys {
            alt: false,
            control: false,
            shift: false,
        }
    }
}

/// Enum representing the keys that can be used in a mapping.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Key {
    Alt(Box<Key>),
    Backspace,
    Char(char),
    Control(Box<Key>),
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
    Shift(Box<Key>),
    Space,
    Tab,
    Up,
}

/// Convert a `Key` a to `String`.
/// Note that the result does not contain < and >.
fn key_to_string(key: &Key) -> String {
    let string =
        match *key {
            Alt(ref key) => return format!("A-{}", key_to_string(&*key)),
            Backspace => "Backspace",
            Char(character) => return character.to_string(),
            Control(ref key) => return format!("C-{}", key_to_string(&*key)),
            Delete => "Delete",
            Down => "Down",
            End => "End",
            Enter => "Enter",
            Escape => "Esc",
            F1 => "F1",
            F2 => "F2",
            F3 => "F3",
            F4 => "F4",
            F5 => "F5",
            F6 => "F6",
            F7 => "F7",
            F8 => "F8",
            F9 => "F9",
            F10 => "F10",
            F11 => "F11",
            F12 => "F12",
            Home => "Home",
            Insert => "Insert",
            Left => "Left",
            PageDown => "PageDown",
            PageUp => "PageUp",
            Right => "Right",
            Shift(ref key) => return format!("S-{}", key_to_string(&*key)),
            Space => "Space",
            Tab => "Tab",
            Up => "Up",
        };
    string.to_string()
}

impl Display for Key {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let string = key_to_string(self);
        let string =
            if let Char(_) = *self {
                string
            }
            else {
                format!("<{}>", string)
            };
        write!(formatter, "{}", string)
    }
}

fn key_constructor(key: Key, constructor_keys: &ConstructorKeys) -> Key {
    let mut ctrl_constructor: fn(Key) -> Key = |key| key;
    if constructor_keys.control {
        ctrl_constructor = |key| Control(Box::new(key));
    }

    let mut shift_constructor: fn(Key) -> Key = |key| key;
    if constructor_keys.shift {
        shift_constructor = |key| Shift(Box::new(key));
    }

    let mut alt_constructor: fn(Key) -> Key = |key| key;
    if constructor_keys.alt {
        alt_constructor = |key| Alt(Box::new(key));
    }

    ctrl_constructor(alt_constructor(shift_constructor(key)))
}

fn parse_key(input: &str, line_num: usize, column_num: usize) -> Result<(Key, usize)> {
    let mut chars = input.chars();
    let result =
        match chars.next() {
            Some('<') => {
                let key: String = chars.take_while(|&character| character != '>').collect();
                if !input.contains('>') {
                    return Err(ParseError::new(
                        Parse,
                        "(none)".to_string(),
                        ">".to_string(),
                        Pos::new(line_num, column_num + input.len())
                    ));

                }
                let mut end = key.clone();
                if end.len() >= 2 && (&end[..2] == "A-" || &end[..2] == "C-" || &end[..2] == "S-") {
                    let mut delta = 0;
                    let mut constructor_keys = ConstructorKeys::new();
                    while end.len() >= 2 && (&end[..2] == "A-" || &end[..2] == "C-" || &end[..2] == "S-") {
                        let new_end = {
                            let (start, new_end) = end.split_at(2);
                            match start {
                                "A-" => constructor_keys.alt = true,
                                "C-" => constructor_keys.control = true,
                                "S-" => constructor_keys.shift = true,
                                _ => unreachable!(),
                            }
                            delta += 2;
                            new_end.to_string()
                        };
                        end = new_end;
                    }

                    let character = end.chars().next().unwrap(); // NOTE: There is at least one character, hence unwrap.
                    let result_special_key = special_key(&end, line_num, column_num + delta, true)
                        .map(|(key, size)| (key_constructor(key, &constructor_keys), size + delta));
                    match result_special_key {
                        Ok(result) => result,
                        Err(error) => {
                            match character {
                                'A' ... 'Z' | 'a' ... 'z' => {
                                    if end.len() == 1 {
                                        (key_constructor(Char(character), &constructor_keys), 5)
                                    }
                                    else {
                                        return Err(ParseError::new(
                                            Parse,
                                            end.to_string(),
                                            "one character".to_string(),
                                            Pos::new(line_num, column_num + 3)
                                        ));
                                    }
                                },
                                _ => return Err(error),
                            }
                        },
                    }
                }
                else {
                    return special_key(&key, line_num, column_num, false);
                }
            },
            Some(character) => {
                let characters = "=+-;!\"'#%&()*,./<>?@[\\]^_{|}~çÇéÉàÀèÈ$";
                match character {
                    'A' ... 'Z' | 'a' ... 'z' => (Char(character), 1),
                    _ if characters.contains(character) => (Char(character), 1),
                    _ =>
                        return Err(ParseError::new(
                            Parse,
                            character.to_string(),
                            "key".to_string(),
                            Pos::new(line_num, column_num)
                        ))
                }
            },
            None => unreachable!() ,
        };
    Ok(result)
}

/// Parse a string into a vector of `Key`s.
pub fn parse_keys(mut input: &str, line_num: usize, column_num: usize) -> Result<Vec<Key>> {
    let mut keys = vec![];
    let mut index = 0;
    while !input.is_empty() {
        let (key, size) = parse_key(input, line_num, column_num + index)?;
        keys.push(key);
        input = &input[size..];
        index += size;
    }
    Ok(keys)
}

/// Parse a special key.
fn special_key(key: &str, line_num: usize, column_num: usize, in_special_key: bool) -> Result<(Key, usize)> {
    let expected =
        if in_special_key {
            "A-Z or special key"
        }
        else {
            "special key"
        };
    let result =
        match key {
            "Backspace" => (Backspace, 11),
            "Delete" => (Delete, 8),
            "Down" => (Down, 6),
            "End" => (End, 5),
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
            "Home" => (Home, 6),
            "Insert" => (Insert, 8),
            "Left" => (Left, 6),
            "PageDown" => (PageDown, 10),
            "PageUp" => (PageUp, 8),
            "Right" => (Right, 7),
            "Space" => (Space, 7),
            "Tab" => (Tab, 5),
            "Up" => (Up, 4),
            _ => return Err(ParseError::new(
                     Parse,
                     key.to_string(),
                     expected.to_string(),
                     Pos::new(line_num, column_num + 1)
                 )),
        };
    Ok(result)
}
