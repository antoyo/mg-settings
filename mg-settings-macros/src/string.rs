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

/// Convert a snake case string to a camel case.
pub fn snake_to_camel(string: &str) -> String {
    let mut chars = string.chars();
    let string =
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        };
    let mut camel = String::new();
    let mut underscore = false;
    for character in string.chars() {
        if character == '_' {
            underscore = true;
        }
        else {
            if underscore {
                camel.push_str(&character.to_uppercase().collect::<String>());
            }
            else {
                camel.push(character);
            }
            underscore = false;
        }
    }
    camel
}

/// Transform a camel case command name to its dashed version.
/// WinOpen is transformed to win-open.
pub fn to_dash_name(name: &str) -> String {
    let mut result = String::new();
    for (index, character) in name.chars().enumerate() {
        let string: String = character.to_lowercase().collect();
        if character.is_uppercase() && index > 0 {
            result.push('-');
        }
        result.push_str(&string);
    }
    result
}
