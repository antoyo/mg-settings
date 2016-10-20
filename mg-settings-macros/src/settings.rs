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
use syn::{Attribute, Body, Ident, MacroInput, Path, VariantData};
use syn::Body::{Enum, Struct};
use syn::MetaItem::Word;
use syn::Ty;

use string::{snake_to_camel, to_dash_name};

/// Expand the required trais for the derive Setting attribute.
pub fn expand_setting_enum(mut ast: MacroInput) -> Tokens {
    let name = ast.ident.clone();
    let mut default = None;

    let mut variant_names = vec![];
    if let Enum(ref mut variants) = ast.body {
        for variant in variants {
            variant_names.push(variant.ident.clone());
            if !variant.attrs.is_empty() {
                for attribute in &variant.attrs {
                    if let &Attribute { value: Word(ref ident), .. } = attribute {
                        if ident == "default" {
                            default = Some(variant.ident.clone());
                        }
                    }
                }
                variant.attrs.clear();
            }
        }
    }
    let choice_names: Vec<_> = variant_names.iter()
        .map(|name| to_dash_name(&name.to_string()))
        .collect();
    let choice_names1 = &choice_names;
    let choice_names2 = &choice_names;

    let from_str_fn = quote! {
        fn from_str(string: &str) -> Result<Self, Self::Err> {
            match string {
                #(#choice_names1 => Ok(#variant_names),)*
                _ => Err(::mg_settings::error::SettingError::UnknownChoice {
                    actual: string.to_string(),
                    expected: vec![#(#choice_names2),*],
                }),
            }
        }
    };

    let default_impl =
        if let Some(ident) = default {
            quote! {
                impl Default for #name {
                    fn default() -> Self {
                        #ident
                    }
                }
            }
        }
        else {
            quote! {
            }
        };

    quote! {
        #[derive(Clone)]
        #ast

        #default_impl

        impl ::std::str::FromStr for #name {
            type Err = ::mg_settings::error::SettingError;

            #from_str_fn
        }
    }
}

/// Expand the required traits for the derive Settings attribute.
pub fn expand_settings_enum(ast: MacroInput) -> Tokens {
    let name = ast.ident.clone();
    let variant_name = Ident::new(format!("{}Variant", name));
    let variant_enum = to_enums(&ast.ident, &variant_name, &ast.body);
    let settings_impl = to_settings_impl(&name, &variant_name, &ast.body);
    quote! {
        #[derive(Default)]
        #ast

        #variant_enum

        #settings_impl
    }
}

/// Check if a type is a custom type (including enum).
fn is_custom_type(ident: &Ident) -> bool {
    match ident.to_string().as_ref() {
        "bool" | "f64" | "i64" | "String" => false,
        _ => true,
    }
}

/// Create the variant enums for getters and setters.
fn to_enums(name: &Ident, variant_name: &Ident, settings_struct: &Body) -> Tokens {
    if let &Struct(VariantData::Struct(ref strct)) = settings_struct {
        let mut field_names = vec![];
        let mut names = vec![];
        let mut types = vec![];
        for field in strct {
            if let Some(ref ident) = field.ident {
                field_names.push(ident);
                let ident = Ident::new(snake_to_camel(&ident.to_string()));
                names.push(ident);
                types.push(field.ty.clone());
            }
        }
        let names1 = &names;
        quote! {
            #[derive(Clone)]
            pub enum #variant_name {
                #(#names1(#types)),*
            }

            impl #name {
                pub fn new() -> #name {
                    #name::default()
                }
            }
        }
    }
    else {
        panic!("Not a struct");
    }
}

/// Create the impl Settings.
fn to_settings_impl(name: &Ident, variant_name: &Ident, settings_struct: &Body) -> Tokens {
    if let &Struct(VariantData::Struct(ref strct)) = settings_struct {
        let mut names = vec![];
        let mut capitalized_names = vec![];
        let mut original_types = vec![];
        let mut types = vec![];
        for field in strct {
            if let Some(ref ident) = field.ident {
                let ident_string = ident.to_string();
                let ident = Ident::new(ident_string.clone());
                names.push(ident);
                let ident = Ident::new(snake_to_camel(&ident_string));
                capitalized_names.push(
                    quote! {
                        #variant_name::#ident
                    });

                if let Ty::Path(_, Path { ref segments, .. }) = field.ty {
                    original_types.push(&segments[0].ident);
                    types.push(to_value_type(&segments[0].ident));
                }
            }
        }
        let string_names: Vec<_> = names.iter()
            .map(|ident| ident.to_string().replace("_", "-"))
            .collect();
        let string_names = &string_names;
        let capitalized_names = &capitalized_names;
        let names1 = &names;
        let variant_exprs = names.iter().zip(original_types.iter())
            .map(|(name, typ)|
                 if is_custom_type(typ) {
                     quote! {
                         ::std::str::FromStr::from_str(&#name)?
                     }
                 }
                 else {
                     quote! { #name }
                 }
            );
        let names2 = &names;
        let names3 = &names;
        let types1 = &types;
        let type_names = types.iter()
            .map(|ident| value_type_to_type(&ident));

        let to_variant_fn_variant = quote! {
            #(#string_names => {
                if let ::mg_settings::Value::#types1(#names1) = value {
                    Ok(#capitalized_names(#variant_exprs))
                }
                else {
                    Err(::mg_settings::error::SettingError::WrongType {
                        actual: value.to_type().to_string(),
                        expected: #type_names.to_string(),
                    })
                }
            },)*
        };

        let to_variant_fn = quote! {
            fn to_variant(name: &str, value: ::mg_settings::Value) -> Result<Self::Variant, ::mg_settings::error::SettingError> {
                match name {
                    #to_variant_fn_variant
                    _ => Err(::mg_settings::error::SettingError::UnknownSetting(name.to_string())),
                }
            }
        };

        quote! {
            impl ::mg_settings::settings::Settings for #name {
                type Variant = #variant_name;

                #to_variant_fn

                fn set_value(&mut self, value: Self::Variant) {
                    match value {
                        #(#capitalized_names(#names1) => {
                            self.#names2 = #names3
                        }),*
                    }
                }
            }
        }
    }
    else {
        panic!("Not a struct");
    }
}

/// Convert a Rust type to a `Value` type.
fn to_value_type(ident: &Ident) -> Ident {
    let value_type =
        match ident.to_string().as_ref() {
            "bool" => "Bool",
            "f64" => "Float",
            "i64" => "Int",
            _ => "Str",
        };
    Ident::new(value_type)
}

/// Convert a `Value` type to a string representation of the type.
fn value_type_to_type(ident: &Ident) -> &str {
    match ident.to_string().as_ref() {
        "Bool" => "bool",
        "Float" => "float",
        "Int" => "integer",
        "Str" => "string",
        ty => panic!("Unknown Value type {}", ty),
    }
}
