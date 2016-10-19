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

//! Settings manager.

use error::SettingError;
use super::Value;

/// Settings manager.
pub trait Settings
    where Self::VariantGet: ToString,
          Self::VariantSet: Clone,
{
    /// The variant enum representing the setting getters.
    type VariantGet;

    /// The variant enum representing the setting setters.
    type VariantSet;

    /// Get a setting value.
    fn get(&self, name: &str) -> Option<Value>;

    /// Set a setting value from its variant.
    fn set_value(&mut self, value: Self::VariantSet);

    /// Convert a name and value to a variant.
    fn to_variant(name: &str, value: Value) -> Result<Self::VariantSet, SettingError>;
}
