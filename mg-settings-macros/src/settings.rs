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
use syn::{Body, Ident, MacroInput, Path, VariantData};
use syn::Body::Struct;
use syn::Ty;

use string::snake_to_camel;

/// Expand the required traits for the derive Settings attribute.
pub fn expand_settings_enum(mut ast: MacroInput) -> Tokens {
    let original_name = ast.ident.clone();
    let variant_name = Ident::new(format!("{}Variant", original_name));
    ast.ident = Ident::new(format!("_{}", original_name));
    let name = ast.ident.clone();
    let variant_enum = to_enums(&original_name, &ast.ident, &variant_name, &ast.body);
    let settings_impl = to_settings_impl(&original_name, &name, &variant_name, &ast.body);
    quote! {
        #[derive(Default)]
        #ast

        #variant_enum

        #settings_impl
    }
}

/// Create the variant enums for getters and setters.
fn to_enums(original_name: &Ident, new_name: &Ident, variant_name: &Ident, settings_struct: &Body) -> Tokens {
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
        let string_names = field_names.iter()
            .map(|ident| ident.to_string().replace("_", "-"));
        let names1 = &names;
        let names2 = &names;
        let qualified_names = names.iter()
            .map(|ident| quote! {
                #original_name::#ident
            });
        quote! {
            #[derive(Clone)]
            pub enum #variant_name {
                #(#names1(#types)),*
            }

            pub enum #original_name {
                #(#names2),*
            }

            impl ::std::fmt::Display for #original_name {
                fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
                    match *self {
                        #(#qualified_names => write!(formatter, #string_names)),*
                    }
                }
            }

            impl #original_name {
                pub fn new() -> #new_name {
                    #new_name::default()
                }
            }
        }
    }
    else {
        panic!("Not a struct");
    }
}

/// Create the impl Settings.
fn to_settings_impl(original_name: &Ident, name: &Ident, variant_name: &Ident, settings_struct: &Body) -> Tokens {
    if let &Struct(VariantData::Struct(ref strct)) = settings_struct {
        let mut names = vec![];
        let mut capitalized_names = vec![];
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
        let names2 = &names;
        let names3 = &names;
        let types1 = &types;
        let type_names = types.iter()
            .map(|ident| value_type_to_type(&ident));

        let to_variant_fn_variant = quote! {
            #(#string_names => {
                if let ::mg_settings::Value::#types1(#names1) = value {
                    Ok(#capitalized_names(#names2))
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
            fn to_variant(name: &str, value: ::mg_settings::Value) -> Result<Self::VariantSet, ::mg_settings::error::SettingError> {
                match name {
                    #to_variant_fn_variant
                    _ => Err(::mg_settings::error::SettingError::UnknownSetting(name.to_string())),
                }
            }
        };

        quote! {
            impl ::mg_settings::settings::Settings for #name {
                type VariantGet = #original_name;
                type VariantSet = #variant_name;

                fn get(&self, name: &str) -> Option<::mg_settings::Value> {
                    match name {
                        #(#string_names => Some(::mg_settings::Value::#types1(self.#names2.clone())),)*
                        _ => None,
                    }
                }

                #to_variant_fn

                fn set_value(&mut self, value: Self::VariantSet) {
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
            "String" => "Str",
            ty => panic!("Unexpected type {}", ty),
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
