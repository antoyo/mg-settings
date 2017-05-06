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

use error::{Error, ParseError, Result};
use error::ErrorType::Parse;
use position::Pos;

pub trait StrExt<'a> {
    fn capitalize(&self) -> String;
    fn rsplit_at(&'a self, index: usize) -> (&'a str, &'a str);
}

impl<'a> StrExt<'a> for &'a str {
    fn capitalize(&self) -> String {
        let mut chars = self.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }

    fn rsplit_at(&'a self, index: usize) -> (&'a str, &'a str) {
        if self.len() > index {
            self.split_at(self.len() - index)
        }
        else {
            ("", "")
        }
    }
}

/// Check if a string is an identifier.
pub fn check_ident(string: String, pos: &Pos) -> Result<String> {
    if string.chars().all(|character| character.is_alphanumeric() || character == '-' || character == '_') {
        if let Some(true) = string.chars().next().map(|character| character.is_alphabetic()) {
            return Ok(string)
        }
    }
    Err(Error::Parse(ParseError::new(Parse, string, "identifier".to_string(), pos.clone())))
}

/// Parse a single word.
pub fn maybe_word(input: &str) -> Option<&str> {
    input.split_whitespace()
        .next()
}

/// Parse a single word.
/// This function assumes there is always at least a word in `input`.
pub fn word(input: &str) -> &str {
    input.split_whitespace()
        .next()
        .unwrap()
}

/// A word found by the words function.
#[derive(Debug, PartialEq)]
pub struct Word<'a> {
    pub index: usize,
    pub word: &'a str,
}

/// Parse `count` words.
pub fn words(input: &str, count: usize) -> Option<Vec<Word>> {
    let mut vec = vec![];
    let mut start_index = 0;
    for (i, character) in input.chars().enumerate() {
        if character.is_whitespace() {
            let word = &input[start_index..i];
            if !word.is_empty() {
                vec.push(Word {
                    index: start_index,
                    word,
                });
            }
            start_index = i + 1;
        }
    }
    let len = input.len();
    if start_index < len {
        vec.push(Word {
            index: start_index,
            word: &input[start_index..len],
        });
    }

    vec.truncate(count);
    if vec.len() == count {
        Some(vec)
    }
    else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{Word, words};

    #[test]
    fn test_words() {
        let word1 = Word {
            index: 0,
            word: "hello",
        };
        let word2 = Word {
            index: 6,
            word: "world",
        };
        assert_eq!(Some(vec![word1, word2]), words("hello world", 2));

        assert_eq!(None, words("hello", 2));

        let word1 = Word {
            index: 0,
            word: "hello",
        };
        let word2 = Word {
            index: 6,
            word: "the",
        };
        assert_eq!(Some(vec![word1, word2]), words("hello the world", 2));

        let word1 = Word {
            index: 1,
            word: "hello",
        };
        let word2 = Word {
            index: 7,
            word: "world",
        };
        assert_eq!(Some(vec![word1, word2]), words(" hello world ", 2));

        let word1 = Word {
            index: 1,
            word: "hello",
        };
        let word2 = Word {
            index: 8,
            word: "world",
        };
        assert_eq!(Some(vec![word1, word2]), words(" hello  world ", 2));
    }
}
