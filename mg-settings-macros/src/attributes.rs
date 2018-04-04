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

use quote::Tokens;
use syn;
use syn::{Attribute, Data, DataEnum, DataStruct, Field, Ident, Variant};
use syn::{MetaList, MetaNameValue};
use syn::Data::{Enum, Struct};
use syn::Lit::Str;
use syn::Meta::{List, NameValue, Word};
use syn::NestedMeta::Meta;
use syn::Type::Path;
use syn::Fields;

use self::VariantInfo::{CommandInfo, SpecialCommandInfo};
use string::to_dash_name;

fn collect_attrs(name: &str, attrs: &[Attribute], hidden: &mut bool, description: &mut String, is_count: &mut bool)
    -> Option<VariantInfo>
{
    for attribute in attrs {
        match attribute.interpret_meta() {
            Some(List(MetaList { ref ident, ref nested, .. })) => {
                match ident.as_ref() {
                    "completion" => {
                        if let Meta(Word(ref arg_ident)) = nested[0] {
                            if arg_ident == "hidden" {
                                *hidden = true;
                            }
                        }
                    },
                    "help" => {
                        if let Meta(NameValue(MetaNameValue { ref ident, ref lit, .. })) = nested[0] {
                            if ident.as_ref() == "text" {
                                if let Str(ref desc) = *lit {
                                    *description = desc.value();
                                }
                            }
                        }
                    },
                    "special_command" => {
                        let mut incremental = false;
                        let mut identifier = None;
                        for arg in nested {
                            if let Meta(ref meta_item) = *arg {
                                match *meta_item {
                                    Word(ref ident) => {
                                        if ident == "incremental" {
                                            incremental = true;
                                        }
                                    },
                                    NameValue(MetaNameValue { ref ident, ref lit, .. }) => {
                                        if ident.as_ref() == "identifier" {
                                            if let Str(ref string) = *lit {
                                                identifier = Some(string.value().chars().next()
                                                                  .expect("identifier should be one character"));
                                            }
                                        }
                                    },
                                    _ => panic!("Unexpected `{:?}`, expecting `incremental`, or `identifier=\"c\"`", meta_item),
                                }
                            }
                        }
                        return Some(SpecialCommandInfo(SpecialCommand {
                            identifier: identifier.expect("identifier is required in #[special_command] attribute"),
                            incremental,
                            name: name.to_string(),
                        }));
                    },
                    _ => (),
                }
            },
            Some(Word(ref ident)) => {
                if ident.as_ref() == "count" {
                    *is_count = true;
                }
            },
            _ => (),
        }
    }
    None
}

fn collect_and_transform_variant(variant: &Variant) -> VariantInfo {
    let mut command = Command::new();
    command.has_argument = variant.fields != Fields::Unit;
    command.name = variant.ident.to_string();
    if let Fields::Unnamed(ref fields) = variant.fields {
        if let Path(syn::TypePath { ref path, .. }) = fields.unnamed[0].ty {
            command.is_optional = path.segments[0].ident.as_ref() == "Option";
        }
    }
    if let Some(special_command) = collect_attrs(&command.name, &variant.attrs, &mut command.hidden,
                                                 &mut command.description, &mut command.is_count)
    {
        special_command
    }
    else {
        CommandInfo(command)
    }
}

fn collect_and_transform_field(field: &Field) -> VariantInfo {
    let mut command = Command::new();
    command.name = field.ident.as_ref().unwrap().to_string();
    if let Some(special_command) = collect_attrs(&command.name, &field.attrs, &mut command.hidden,
                                                 &mut command.description, &mut command.is_count)
    {
        special_command
    }
    else {
        CommandInfo(command)
    }
}

#[derive(Debug)]
pub struct Command {
    pub description: String,
    pub has_argument: bool,
    pub hidden: bool,
    pub is_count: bool,
    pub is_optional: bool,
    pub name: String,
}

impl Command {
    fn new() -> Self {
        Command {
            description: String::new(),
            has_argument: false,
            hidden: false,
            is_count: false,
            is_optional: false,
            name: String::new(),
        }
    }
}

#[derive(Debug)]
pub struct SpecialCommand {
    pub identifier: char,
    pub incremental: bool,
    pub name: String,
}

/// Struct holding metadata information about all the variants.
#[derive(Debug)]
pub enum VariantInfo {
    CommandInfo(Command),
    SpecialCommandInfo(SpecialCommand),
}

/// Create the EnumMetaData impl.
pub fn to_metadata_impl(name: &Ident, body: &Data) -> (Tokens, Vec<VariantInfo>) {
    let variant_infos = transform_enum(body);
    let tokens = {
        let metadata = variant_infos.iter()
            .filter_map(|info| if let CommandInfo(ref command) = *info {
                let name = to_dash_name(&command.name).replace('_', "-");
                let is_hidden = command.hidden || command.is_count;
                let description = &command.description;
                let metadata = quote! {
                    (#name.to_string(), ::mg_settings::MetaData {
                        completion_hidden: #is_hidden,
                        help_text: #description.to_string(),
                        is_special_command: false,
                    })
                };
                Some(metadata)
            }
            else {
                None
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
    (tokens, variant_infos)
}

/// Remove the attributes from the variants and return the metadata gathered from the attributes.
pub fn transform_enum(item: &Data) -> Vec<VariantInfo> {
    let mut variant_infos = vec![];
    match *item {
        Enum(DataEnum{ ref variants, .. }) => {
            for variant in variants {
                variant_infos.push(collect_and_transform_variant(variant));
            }
        },
        Struct(DataStruct { ref fields, .. }) => {
            for field in fields {
                variant_infos.push(collect_and_transform_field(field));
            }
        },
        _ => (),
    }
    variant_infos
}
