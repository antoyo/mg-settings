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
use syn::{Body, Ident, MacroInput, VariantData};

use attributes::to_metadata_impl;
use attributes::VariantInfo::{self, CommandInfo, SpecialCommandInfo};
use string::to_dash_name;

/// Expand the required traits for the derive Commands attribute.
pub fn expand_commands_enum(mut ast: MacroInput) -> Tokens {
    let name = &ast.ident;
    let (metadata_impl, variant_infos) = to_metadata_impl(name, &mut ast.body);
    let special_command_impl = to_special_command_impl(name, &variant_infos);
    let mut variant_values = vec![];
    let mut variant_names_with_argument = vec![];
    let mut variant_names_without_argument = vec![];
    let mut variant_names = vec![];
    for info in &variant_infos {
        if let CommandInfo(ref command) = *info {
            let command_name = &command.name;
            let dash_name = to_dash_name(command_name);
            variant_names.push(dash_name.clone());
            if command.has_argument {
                variant_names_with_argument.push(dash_name.clone());
            }
            else {
                variant_names_without_argument.push(dash_name);
            }
            let ident = Ident::new(command_name.as_ref());
            let arg_ident = Ident::new("argument");
            let value =
                if command.has_argument {
                    quote! {
                        #name::#ident(#arg_ident.to_string())
                    }
                }
                else {
                    quote! {
                        #name::#ident
                    }
                };
            variant_values.push(value);
        }
    }
    let variant_names = &variant_names;
    let fn_has_argument = quote!{
        fn has_argument(variant: &str) -> ::std::result::Result<bool, String> {
            match variant {
                #(#variant_names_with_argument)|* => Ok(true),
                #(#variant_names_without_argument)|* => Ok(false),
                _ => Err(format!("unknown command {}", variant)),
            }
        }
    };
    let clone = derive_clone(&ast);
    quote! {
        impl ::mg_settings::EnumFromStr for #name {
            fn create(variant: &str, argument: &str) -> ::std::result::Result<#name, String> {
                match variant {
                    #(#variant_names => Ok(#variant_values),)*
                    _ => Err(format!("unknown command {}", variant)),
                }
            }

            #fn_has_argument
        }

        #clone

        #metadata_impl
        #special_command_impl
    }
}

fn derive_clone(ast: &MacroInput) -> Tokens {
    let name = &ast.ident;

    if let Body::Enum(ref variants) = ast.body {
        let variant_idents_values: Vec<_> = variants.iter().map(|variant| {
            let has_value =
                if let VariantData::Tuple(_) = variant.data {
                    true
                }
                else {
                    false
                };
            (&variant.ident, has_value)
        }).collect();
        let variant_patterns = variant_idents_values.iter().map(|&(ref ident, has_value)| {
            if has_value {
                quote! {
                    #name::#ident(ref value)
                }
            }
            else {
                quote! {
                    #name::#ident
                }
            }
        });
        let variant_values = variant_idents_values.iter().map(|&(ref ident, has_value)| {
            if has_value {
                quote! {
                    #name::#ident(value.clone())
                }
            }
            else {
                quote! {
                    #name::#ident
                }
            }
        });

        quote! {
            impl Clone for #name {
                fn clone(&self) -> Self {
                    match *self {
                        #(#variant_patterns => #variant_values,)*
                    }
                }
            }
        }
    }
    else {
        panic!("Expected enum");
    }
}

fn to_special_command_impl(name: &Ident, variant_infos: &[VariantInfo]) -> Tokens {
    let mut identifiers = vec![];
    let mut incremental_identifiers = vec![];
    let mut to_commands = vec![];
    for info in variant_infos {
        if let SpecialCommandInfo(ref command) = *info {
            let identifier = command.identifier;
            identifiers.push(identifier);
            if command.incremental {
                incremental_identifiers.push(identifier);
            }
            let command = Ident::new(command.name.as_ref());
            to_commands.push(quote! {
                #identifier => Ok(#name::#command(input.to_string())),
            });
        }
    }
    let true_identifiers_tokens = gen_list_to_true(&identifiers);
    let true_incremental_identifiers_tokens = gen_list_to_true(&incremental_identifiers);
    quote! {
        impl ::mg_settings::SpecialCommand for #name {
            #[allow(unused_variables)]
            fn identifier_to_command(identifier: char, input: &str) -> ::std::result::Result<Self, String> {
                match identifier {
                    #(#to_commands)*
                    _ => Err(format!("unknown identifier {}", identifier)),
                }
            }

            fn is_identifier(character: char) -> bool {
                match character {
                    #true_identifiers_tokens
                    _ => false,
                }
            }

            fn is_incremental(identifier: char) -> bool {
                match identifier {
                    #true_incremental_identifiers_tokens
                    _ => false,
                }
            }
        }
    }
}

fn gen_list_to_true(identifiers: &[char]) -> Tokens {
    if identifiers.is_empty() {
        quote! {
        }
    }
    else {
        quote! {
            #( #identifiers )|* => true,
        }
    }
}
