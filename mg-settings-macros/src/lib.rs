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

#![recursion_limit="128"]

extern crate env_logger;
#[macro_use]
extern crate log;
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

mod attributes;
mod commands;
mod settings;
mod string;

use std::env;

use env_logger::LogBuilder;
use log::LogRecord;
use proc_macro::TokenStream;

use commands::expand_commands_enum;
use settings::{expand_setting_enum, expand_settings_enum};

fn init_logger() {
    let format = |record: &LogRecord| {
        record.args().to_string()
    };
    let mut builder = LogBuilder::new();
    builder.format(format);
    if let Ok(rust_log) = env::var("RUST_LOG") {
        builder.parse(&rust_log);
    }
    builder.init().ok();
}

#[proc_macro_derive(Commands, attributes(completion, count, help, special_command))]
/// Derive Commands.
pub fn commands(input: TokenStream) -> TokenStream {
    init_logger();
    let ast = syn::parse(input).unwrap();
    let expanded = expand_commands_enum(ast);
    warn!("{}", expanded.to_string());
    expanded.into()
}

#[proc_macro_derive(Setting, attributes(default))]
/// Derive Setting.
pub fn setting(input: TokenStream) -> TokenStream {
    init_logger();
    let ast = syn::parse(input).unwrap();
    let expanded = expand_setting_enum(ast);
    warn!("{}", expanded.to_string());
    expanded.into()
}

#[proc_macro_derive(Settings, attributes(help))]
/// Derive Settings.
pub fn settings(input: TokenStream) -> TokenStream {
    init_logger();
    let ast = syn::parse(input).unwrap();
    let expanded = expand_settings_enum(ast);
    warn!("{}", expanded.to_string());
    expanded.into()
}
