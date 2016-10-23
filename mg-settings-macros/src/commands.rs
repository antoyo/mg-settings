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
use syn::{Ident, MacroInput};

use attributes::to_metadata_impl;
use string::to_dash_name;

/// Expand the required traits for the derive Commands attribute.
pub fn expand_commands_enum(mut ast: MacroInput) -> Tokens {
    let name = &ast.ident;
    let (metadata_impl, variant_info) = to_metadata_impl(name, &mut ast.body);
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

        #metadata_impl
    }
}
