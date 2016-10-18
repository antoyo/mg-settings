#![feature(proc_macro, proc_macro_lib)]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use quote::Tokens;
use syn::{Attribute, Body, Ident, MacroInput};
use syn::Body::Enum;
use syn::Lit::Str;
use syn::MetaItem::{List, NameValue, Word};
use syn::VariantData::Unit;
use proc_macro::TokenStream;

/// Struct holding metadata information about all the variants.
struct VariantInfo {
    descriptions: Vec<String>,
    has_argument: Vec<bool>,
    hidden: Vec<bool>,
    names: Vec<String>,
}

#[proc_macro_derive(Commands)]
/// Derive Commands.
pub fn commands(input: TokenStream) -> TokenStream {
    let source = input.to_string();
    let ast = syn::parse_macro_input(&source).unwrap();
    let expanded = expand_commands_enum(ast);
    expanded.to_string().parse().unwrap()
}

/// Expand the required traits for the derive Commands attribute.
fn expand_commands_enum(mut ast: MacroInput) -> Tokens {
    let variant_info = transform_enum(&mut ast.body);
    let variant_values = variant_info.names.iter()
        .zip(&variant_info.has_argument)
        .map(|(name, &has_argument)| {
            let ident = Ident::new(name.as_ref());
            let arg_ident = Ident::new("argument");
            if has_argument {
                quote! {
                    #ident(#arg_ident.to_string())
                }
            }
            else {
                quote! {
                    #ident
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
    let name = ast.ident.clone();
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

/// Transform a camel case command name to its dashed version.
/// WinOpen is transformed to win-open.
fn to_dash_name(name: &str) -> String {
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
