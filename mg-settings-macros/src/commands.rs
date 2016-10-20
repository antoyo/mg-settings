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
use syn::{Attribute, Body, Ident, MacroInput};
use syn::Body::Enum;
use syn::Lit::Str;
use syn::MetaItem::{List, NameValue, Word};
use syn::VariantData::Unit;

use string::to_dash_name;

/// Struct holding metadata information about all the variants.
struct VariantInfo {
    descriptions: Vec<String>,
    has_argument: Vec<bool>,
    hidden: Vec<bool>,
    names: Vec<String>,
}

/// Expand the required traits for the derive Commands attribute.
pub fn expand_commands_enum(mut ast: MacroInput) -> Tokens {
    let variant_info = transform_enum(&mut ast.body);
    let name = ast.ident.clone();
    let variant_values = variant_info.names.iter()
        .zip(&variant_info.has_argument)
        .map(|(variant_name, &has_argument)| {
            let ident = Ident::new(variant_name.as_ref());
            let arg_ident = Ident::new("argument");
            if has_argument {
                quote! {
                    #name::#ident(#arg_ident.to_string())
                }
            }
            else {
                quote! {
                    #name::#ident
                }
            }
        });
    let variant_names: Vec<_> = variant_info.names.iter()
        .map(|name| to_dash_name(&name))
        .collect();
    let metadata = variant_names.iter()
        .zip(&variant_info.hidden)
        .zip(&variant_info.descriptions)
        .map(|((name, &is_hidden), description)| {
            let name = to_dash_name(name);
            quote! {
                (#name.to_string(), ::mg_settings::CommandMetaData {
                    completion_hidden: #is_hidden,
                    help_text: #description.to_string(),
                    is_special_command: false,
                })
            }
        });
    let variant_names = &variant_names;
    let variant_has_argument = &variant_info.has_argument;
    quote! {
        #ast

        impl ::mg_settings::EnumFromStr for #name {
            fn create(variant: &str, argument: &str) -> ::std::result::Result<#name, String> {
                match variant {
                    #(#variant_names => Ok(#variant_values),)*
                    _ => Err(format!("unknown command {}", variant)),
                }
            }

            fn has_argument(variant: &str) -> ::std::result::Result<bool, String> {
                match variant {
                    #(#variant_names => Ok(#variant_has_argument),)*
                    _ => Err(format!("unknown command {}", variant)),
                }
            }
        }

        impl ::mg_settings::EnumMetaData for #name {
            fn get_metadata() -> ::std::collections::HashMap<String, ::mg_settings::CommandMetaData> {
                let mut vec = vec![#(#metadata),*];
                let iter = vec.drain(..);
                iter.collect()
            }
        }
    }
}

/// Remove the attributes from the variants and return the metadata gathered from the attributes.
fn transform_enum(item: &mut Body) -> VariantInfo {
    let mut descriptions = vec![];
    let mut has_argument = vec![];
    let mut hidden_variants = vec![];
    let mut variant_names = vec![];
    if let Enum(ref mut variants) = *item {
        for variant in variants {
            has_argument.push(variant.data != Unit);
            variant_names.push(variant.ident.to_string());
            let mut help = String::new();
            let mut hidden = false;
            if !variant.attrs.is_empty() {
                for attribute in &variant.attrs {
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
                variant.attrs.clear();
            }
            descriptions.push(help);
            hidden_variants.push(hidden);
        }
    }
    VariantInfo {
        descriptions: descriptions,
        has_argument: has_argument,
        hidden: hidden_variants,
        names: variant_names,
    }
}
