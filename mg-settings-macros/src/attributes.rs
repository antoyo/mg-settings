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

use quote::Tokens;
use syn::{Attribute, Body, Ident};
use syn::Body::{Enum, Struct};
use syn::Lit::Str;
use syn::MetaItem::{List, NameValue, Word};
use syn::VariantData::{self, Unit};

use string::to_dash_name;

macro_rules! push_data {
    (true, $has_argument:expr, $item_names:expr, $item:expr) => {
        $has_argument.push($item.data != Unit);
        $item_names.push($item.ident.to_string());
    };
    (false, $has_argument:expr, $item_names:expr, $item:expr) => {
        $item_names.push($item.ident.as_ref().unwrap().to_string());
    };
}

macro_rules! collect_and_transform {
    ($item:expr, $is_variant:tt, $has_argument:expr, $item_names:expr, $descriptions:expr, $hidden_items:expr) => {
        push_data!($is_variant, $has_argument, $item_names, $item);
        let mut help = String::new();
        let mut hidden = false;
        if !$item.attrs.is_empty() {
            for attribute in &$item.attrs {
                if let &Attribute { value: List(ref ident, ref args), .. } = attribute {
                    match ident.as_ref() {
                        "completion" => {
                            if let Word(ref arg_ident) = args[0] {
                                if arg_ident == "hidden" {
                                    hidden = true;
                                }
                            }
                        },
                        "help" => {
                            if let NameValue(ref arg_ident, ref value) = args[0] {
                                if arg_ident == "text" {
                                    if let &Str(ref description, _) = value {
                                        help = description.clone();
                                    }
                                }
                            }
                        },
                        _ => (),
                    }
                }
            }
        }
        $descriptions.push(help);
        $hidden_items.push(hidden);
    };
}

/// Struct holding metadata information about all the variants.
#[derive(Debug)]
pub struct VariantInfo {
    pub descriptions: Vec<String>,
    pub has_argument: Vec<bool>,
    pub hidden: Vec<bool>,
    pub names: Vec<String>,
}

/// Create the EnumMetaData impl.
pub fn to_metadata_impl(name: &Ident, body: &Body) -> (Tokens, VariantInfo) {
    let variant_info = transform_enum(body);
    let variant_names: Vec<_> = variant_info.names.iter()
        .map(|name| to_dash_name(&name))
        .collect();
    let tokens = {
        let metadata =
            variant_names.iter()
                .zip(&variant_info.hidden)
                .zip(&variant_info.descriptions)
                .map(|((name, &is_hidden), description)| {
                    let name = to_dash_name(name).replace('_', "-");
                    quote! {
                        (#name.to_string(), ::mg_settings::MetaData {
                            completion_hidden: #is_hidden,
                            help_text: #description.to_string(),
                            is_special_command: false,
                        })
                    }
                });
        quote! {
            impl ::mg_settings::EnumMetaData for #name {
                fn get_metadata() -> ::std::collections::HashMap<String, ::mg_settings::MetaData> {
                    let mut vec = vec![#(#metadata),*];
                    let iter = vec.drain(..);
                    iter.collect()
                }
            }
        }
    };
    (tokens, variant_info)
}

/// Remove the attributes from the variants and return the metadata gathered from the attributes.
pub fn transform_enum(item: &Body) -> VariantInfo {
    let mut descriptions = vec![];
    let mut has_argument = vec![];
    let mut hidden_items = vec![];
    let mut item_names = vec![];
    match *item {
        Enum(ref variants) => {
            for variant in variants {
                collect_and_transform!(variant, true, has_argument, item_names, descriptions, hidden_items);
            }
        },
        Struct(VariantData::Struct(ref fields)) => {
            for field in fields {
                collect_and_transform!(field, false, has_argument, item_names, descriptions, hidden_items);
            }
        },
        _ => (),
    }
    VariantInfo {
        descriptions: descriptions,
        has_argument: has_argument,
        hidden: hidden_items,
        names: item_names,
    }
}
