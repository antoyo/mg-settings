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
use syn::{Attribute, Body, Field, Ident, Variant};
use syn::Body::{Enum, Struct};
use syn::Lit::Str;
use syn::MetaItem::{List, NameValue, Word};
use syn::NestedMetaItem::MetaItem;
use syn::Ty::Path;
use syn::VariantData::{self, Unit};

use self::VariantInfo::{CommandInfo, SpecialCommandInfo};
use string::to_dash_name;

fn collect_attrs(name: &str, attrs: &[Attribute], hidden: &mut bool, description: &mut String, is_count: &mut bool)
    -> Option<VariantInfo>
{
    for attribute in attrs {
        match *attribute {
            Attribute { value: List(ref ident, ref args), .. } => {
                match ident.as_ref() {
                    "completion" => {
                        if let MetaItem(Word(ref arg_ident)) = args[0] {
                            if arg_ident == "hidden" {
                                *hidden = true;
                            }
                        }
                    },
                    "help" => {
                        if let MetaItem(NameValue(ref arg_ident, ref value)) = args[0] {
                            if arg_ident == "text" {
                                if let Str(ref desc, _) = *value {
                                    *description = desc.clone();
                                }
                            }
                        }
                    },
                    "special_command" => {
                        let mut incremental = false;
                        let mut identifier = None;
                        for arg in args {
                            if let MetaItem(ref meta_item) = *arg {
                                match *meta_item {
                                    Word(ref ident) => {
                                        if ident == "incremental" {
                                            incremental = true;
                                        }
                                    },
                                    NameValue(ref ident, ref value) => {
                                        if ident == "identifier" {
                                            if let Str(ref string, _) = *value {
                                                identifier = Some(string.chars().next()
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
            Attribute { value: Word(ref ident), .. } => {
                if ident == "count" {
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
    command.has_argument = variant.data != Unit;
    command.name = variant.ident.to_string();
    if let VariantData::Tuple(ref fields) = variant.data {
        if let Path(_, syn::Path { ref segments, .. }) = fields[0].ty {
            command.is_optional = segments[0].ident == "Option";
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
pub fn to_metadata_impl(name: &Ident, body: &Body) -> (Tokens, Vec<VariantInfo>) {
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
pub fn transform_enum(item: &Body) -> Vec<VariantInfo> {
    let mut variant_infos = vec![];
    match *item {
        Enum(ref variants) => {
            for variant in variants {
                variant_infos.push(collect_and_transform_variant(variant));
            }
        },
        Struct(VariantData::Struct(ref fields)) => {
            for field in fields {
                variant_infos.push(collect_and_transform_field(field));
            }
        },
        _ => (),
    }
    variant_infos
}
