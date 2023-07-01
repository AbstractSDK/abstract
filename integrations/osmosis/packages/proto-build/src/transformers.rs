use std::collections::HashMap;
use std::path::Path;

use heck::ToSnakeCase;
use heck::ToUpperCamelCase;
use proc_macro2::{Group, TokenStream as TokenStream2, TokenTree};
use prost_types::{
    DescriptorProto, EnumDescriptorProto, FileDescriptorSet, ServiceDescriptorProto,
};
use quote::{format_ident, quote};
use regex::Regex;
use syn::ItemEnum;
use syn::ItemMod;
use syn::{parse_quote, Attribute, Fields, Ident, Item, ItemStruct, Type};

/// Regex substitutions to apply to the prost-generated output
pub const REPLACEMENTS: &[(&str, &str)] = &[
    // Use `tendermint-proto` proto definitions
    ("(super::)+tendermint", "tendermint_proto"),
    // Feature-gate gRPC client modules
    (
        "/// Generated client implementations.",
        "/// Generated client implementations.\n\
             #[cfg(feature = \"grpc\")]\n\
             #[cfg_attr(docsrs, doc(cfg(feature = \"grpc\")))]",
    ),
    // Feature-gate gRPC client impls which use `tonic::transport`
    (
        "impl (.+)Client<tonic::transport::Channel>",
        "#[cfg(feature = \"grpc-transport\")]\n    \
             #[cfg_attr(docsrs, doc(cfg(feature = \"grpc-transport\")))]\n    \
             impl ${1}Client<tonic::transport::Channel>",
    ),
];

pub fn add_derive_eq(mut attr: Attribute) -> Attribute {
    // find derive attribute
    if attr.path.is_ident("derive") {
        attr.tokens = attr
            .tokens
            .into_iter()
            .map(|token_tree| {
                match token_tree {
                    // with group token stream, which is `#[derive( ... )]`
                    TokenTree::Group(group) => {
                        let has_ident = |ident_str: &str| {
                            group.stream().into_iter().any(|token| match token {
                                TokenTree::Ident(ident) => ident == format_ident!("{}", ident_str),
                                _ => false,
                            })
                        };

                        // if does not have both PartialEq and Eq
                        let stream = if !(has_ident("PartialEq") && has_ident("Eq")) {
                            // construct new token stream
                            group
                                .stream()
                                .into_iter()
                                .flat_map(|token| {
                                    match token {
                                        // if there exist `PartialEq` in derive attr
                                        TokenTree::Ident(ident) => {
                                            if ident == format_ident!("PartialEq") {
                                                // expand token stream in group with `#[derive( ..., PartialEq, ... )]` to ``#[derive( ..., PartialEq, Eq, ... )]``
                                                let expanded_token_stream: TokenStream2 =
                                                    syn::parse_quote!(PartialEq, Eq);
                                                expanded_token_stream.into_iter().collect()
                                            } else {
                                                vec![TokenTree::Ident(ident)]
                                            }
                                        }
                                        tt => vec![tt],
                                    }
                                })
                                .collect()
                        } else {
                            group.stream()
                        };

                        TokenTree::Group(Group::new(group.delimiter(), stream))
                    }
                    _ => token_tree,
                }
            })
            .collect();
        attr
    } else {
        attr
    }
}

pub fn add_derive_eq_struct(s: &ItemStruct) -> ItemStruct {
    let mut item_struct = s.clone();
    item_struct.attrs = item_struct.attrs.into_iter().map(add_derive_eq).collect();

    item_struct
}

pub fn add_derive_eq_enum(s: &ItemEnum) -> ItemEnum {
    let mut item_enum = s.clone();
    item_enum.attrs = item_enum.attrs.into_iter().map(add_derive_eq).collect();

    item_enum
}

pub fn append_attrs_struct(
    src: &Path,
    s: &ItemStruct,
    descriptor: &FileDescriptorSet,
) -> ItemStruct {
    let mut s = s.clone();
    let query_services = extract_query_services(descriptor);
    let type_url = get_type_url(src, &s.ident, descriptor);

    let deprecated = get_deprecation(src, &s.ident, descriptor);

    s.attrs.append(&mut vec![
        syn::parse_quote! { #[derive(::serde::Serialize, ::serde::Deserialize, ::schemars::JsonSchema, CosmwasmExt)] },
        syn::parse_quote! { #[proto_message(type_url = #type_url)] },
    ]);

    if let Some(attr) = get_query_attr(src, &s.ident, &query_services) {
        s.attrs.append(&mut vec![attr])
    }

    if deprecated {
        s.attrs
            .append(&mut vec![syn::parse_quote! { #[deprecated] }]);
    }

    s
}

pub fn append_attrs_enum(src: &Path, e: &ItemEnum, descriptor: &FileDescriptorSet) -> ItemEnum {
    let mut e = e.clone();
    let deprecated = get_deprecation(src, &e.ident, descriptor);

    e.attrs.append(&mut vec![
        syn::parse_quote! { #[derive(::serde::Serialize, ::serde::Deserialize, ::schemars::JsonSchema)] },
    ]);

    if deprecated {
        e.attrs
            .append(&mut vec![syn::parse_quote! { #[deprecated] }]);
    }

    e
}

pub fn allow_serde_int_as_str(s: ItemStruct) -> ItemStruct {
    let fields_vec = s
        .fields
        .clone()
        .into_iter()
        .map(|mut field| {
            let int_types = vec![
                parse_quote!(i8),
                parse_quote!(i16),
                parse_quote!(i32),
                parse_quote!(i64),
                parse_quote!(i128),
                parse_quote!(isize),
                parse_quote!(u8),
                parse_quote!(u16),
                parse_quote!(u32),
                parse_quote!(u64),
                parse_quote!(u128),
                parse_quote!(usize),
            ];

            if int_types.contains(&field.ty) {
                let from_str: syn::Attribute = parse_quote! {
                    #[serde(
                        serialize_with = "crate::serde::as_str::serialize",
                        deserialize_with = "crate::serde::as_str::deserialize"
                    )]
                };
                field.attrs.append(&mut vec![from_str]);
                field
            } else {
                field
            }
        })
        .collect::<Vec<syn::Field>>();

    let fields_named: syn::FieldsNamed = parse_quote! {
        { #(#fields_vec,)* }
    };
    let fields = syn::Fields::Named(fields_named);

    syn::ItemStruct { fields, ..s }
}

/// some of proto's fields in osmosis' modules are named `ID` but prost generates `id` field
/// this function adds `#[serde(alias = "ID")]` to the `id` field
/// so that serde can deserialize `ID` field to `id` field.
/// This is required because the `ID` field is used in the query response and is serialized as json.
pub fn serde_alias_id_with_uppercased(s: ItemStruct) -> ItemStruct {
    let fields_vec = s
        .fields
        .clone()
        .into_iter()
        .map(|mut field| {
            if let Some(ident) = &field.ident {
                let ident_str = ident.to_string();
                if ident_str == "id" {
                    let serde_alias_id: syn::Attribute = parse_quote! {
                        #[serde(alias = "ID")]
                    };
                    field.attrs.append(&mut vec![serde_alias_id]);
                    field
                } else if ident_str.contains("_id") {
                    let ident_str = ident_str.replace("_id", "ID");
                    let serde_alias_id: syn::Attribute = parse_quote! {
                        #[serde(alias = #ident_str)]
                    };
                    field.attrs.append(&mut vec![serde_alias_id]);
                    field
                } else {
                    field
                }
            } else {
                field
            }
        })
        .collect::<Vec<syn::Field>>();

    let fields_named: syn::FieldsNamed = parse_quote! {
        { #(#fields_vec,)* }
    };
    let fields = syn::Fields::Named(fields_named);

    syn::ItemStruct { fields, ..s }
}
// ====== helpers ======

fn get_query_attr(
    src: &Path,
    ident: &Ident,
    query_services: &HashMap<String, ServiceDescriptorProto>,
) -> Option<Attribute> {
    let package = src.file_stem().unwrap().to_str().unwrap();
    let service = query_services.get(package);

    let method = service?.method.iter().find(|m| {
        let input_type = m.input_type.clone().unwrap();
        let input_type = input_type.split('.').last().unwrap();
        *ident == input_type.to_upper_camel_case()
    });

    let method_name = method?.name.clone().unwrap();
    let response_type = method?.output_type.clone().unwrap();
    let response_type = response_type.split('.').last().unwrap();
    let response_type = format_ident!("{}", response_type.to_upper_camel_case());

    let path = format!("/{}.Query/{}", package, method_name);
    Some(syn::parse_quote! { #[proto_query(path = #path, response_type = #response_type)] })
}

fn get_type_url(src: &Path, ident: &Ident, descriptor: &FileDescriptorSet) -> String {
    let type_path = src.file_stem().unwrap().to_str().unwrap();
    let init_path = "";

    let name: Option<String> = descriptor
        .file
        .clone()
        .into_iter()
        .filter(|f| f.package.to_owned().unwrap() == type_path)
        .flat_map(|f| {
            let target = ident.to_string();
            vec![
                extract_type_path_from_enum(&target, &f.enum_type, init_path),
                extract_type_path_from_descriptor(&target, &f.message_type, init_path),
            ]
        })
        .filter(|r| r.is_some())
        .take(1)
        .collect();

    format!("/{}.{}", type_path, name.unwrap())
}

fn get_deprecation(src: &Path, ident: &Ident, descriptor: &FileDescriptorSet) -> bool {
    let type_path = src.file_stem().unwrap().to_str().unwrap();

    let deprecation: Option<bool> = descriptor
        .file
        .clone()
        .into_iter()
        .filter(|f| f.package.to_owned().unwrap() == type_path)
        .flat_map(|f| {
            let target = ident.to_string();
            vec![
                extract_deprecation_from_enum(&target, &f.enum_type),
                extract_deprecation_from_descriptor(&target, &f.message_type),
            ]
        })
        .find(|r| r.is_some())
        .flatten();

    deprecation.unwrap_or(false)
}

fn extract_deprecation_from_descriptor(
    target: &str,
    message_type: &[DescriptorProto],
) -> Option<bool> {
    message_type.iter().find_map(|descriptor| {
        let message_name = descriptor.name.to_owned().unwrap();

        if message_name.to_upper_camel_case() == target {
            descriptor.clone().options?.deprecated
        } else if let Some(deprecated) =
            extract_deprecation_from_descriptor(target, &descriptor.nested_type)
        {
            Some(deprecated)
        } else {
            extract_deprecation_from_enum(target, &descriptor.enum_type)
        }
    })
}

fn extract_deprecation_from_enum(target: &str, enum_type: &[EnumDescriptorProto]) -> Option<bool> {
    enum_type
        .iter()
        .find(|e| e.name.to_owned().unwrap().to_upper_camel_case() == target)
        .and_then(|e| e.clone().options?.deprecated)
}

fn extract_type_path_from_descriptor(
    target: &str,
    message_type: &[DescriptorProto],
    path: &str,
) -> Option<String> {
    message_type.iter().find_map(|descriptor| {
        let message_name = descriptor.name.to_owned().unwrap();

        if message_name.to_upper_camel_case() == target {
            Some(append_type_path(path, &message_name))
        } else if let Some(message_name) = extract_type_path_from_descriptor(
            target,
            &descriptor.nested_type,
            &append_type_path(path, &message_name),
        ) {
            Some(message_name)
        } else {
            extract_type_path_from_enum(
                target,
                &descriptor.enum_type,
                &append_type_path(path, &message_name),
            )
        }
    })
}

fn extract_type_path_from_enum(
    target: &str,
    enum_type: &[EnumDescriptorProto],
    path: &str,
) -> Option<String> {
    enum_type
        .iter()
        .find(|e| e.name.to_owned().unwrap().to_upper_camel_case() == target)
        .map(|e| append_type_path(path, &e.name.to_owned().unwrap()))
}

pub fn extract_query_services(
    descriptor: &FileDescriptorSet,
) -> HashMap<String, ServiceDescriptorProto> {
    descriptor
        .clone()
        .file
        .into_iter()
        .filter_map(|f| {
            let service = f
                .service
                .into_iter()
                .find(|s| s.name == Some("Query".to_string()));

            if let Some(service) = service {
                Some((
                    f.package.expect("Missing package name in file descriptor"),
                    service,
                ))
            } else {
                None
            }
        })
        .collect()
}

fn append_type_path(path: &str, name: &str) -> String {
    if path.is_empty() {
        name.to_string()
    } else {
        format!("{}.{}", path, name)
    }
}

pub fn append_querier(
    items: Vec<Item>,
    src: &Path,
    nested_mod: bool,
    descriptor: &FileDescriptorSet,
) -> Vec<Item> {
    let package = src.file_stem().unwrap().to_str().unwrap();
    let re = Regex::new(r"([^.]*)(\.v\d+(beta\d+)?)?$").unwrap();

    let package_stem = re.captures(package).unwrap().get(1).unwrap().as_str();

    let querier_wrapper_ident = format_ident!("{}Querier", &package_stem.to_upper_camel_case());

    let query_services = extract_query_services(descriptor);
    let query_fns = query_services.get(package).map(|service| service.method.iter().map(|method_desc| {
        if nested_mod {
            return quote! {};
        }

        let deprecated = method_desc.clone().options.map(|opt| opt.deprecated.unwrap_or(false) ).unwrap_or(false);
        let deprecated_macro = if deprecated {
            quote!(#[deprecated])
        } else {
            quote!()
        };

        let method_desc = method_desc.clone();

        let name = format_ident!("{}", method_desc.name.unwrap().as_str().to_snake_case());
        let req_type = format_ident!("{}", method_desc.input_type.unwrap().split('.').last().unwrap().to_string().to_upper_camel_case());
        let res_type = format_ident!("{}", method_desc.output_type.unwrap().split('.').last().unwrap().to_string().to_upper_camel_case());

        let req_args = items.clone().into_iter()
            .find_map(|item| match item {
                Item::Struct(s) => {
                    if s.ident == req_type {
                        match s.fields {
                            Fields::Named(fields_named) => {
                                Some(fields_named.named)
                            }
                            _ => None
                        }
                    } else {
                        None
                    }
                }
                _ => None
            });

        let arg_idents = req_args.clone().unwrap().into_iter().map(|arg| arg.ident.unwrap()).collect::<Vec<Ident>>();
        let arg_ty = req_args.unwrap().into_iter().map(|arg| arg.ty).collect::<Vec<Type>>();

        quote! {
          #deprecated_macro
          pub fn #name( &self, #(#arg_idents : #arg_ty),* ) -> Result<#res_type, cosmwasm_std::StdError> {
            #req_type { #(#arg_idents),* }.query(self.querier)
          }
        }
    }).collect::<Vec<TokenStream2>>());

    let querier = if let Some(query_fns) = query_fns {
        if !nested_mod {
            vec![
                parse_quote! {
                  pub struct #querier_wrapper_ident<'a, Q: cosmwasm_std::CustomQuery> {
                      querier: &'a cosmwasm_std::QuerierWrapper<'a, Q>,
                  }
                },
                parse_quote! {
                  impl<'a, Q: cosmwasm_std::CustomQuery> #querier_wrapper_ident<'a, Q> {
                      pub fn new(querier: &'a cosmwasm_std::QuerierWrapper<'a, Q>) -> Self {
                    Self { querier }
                    }
                    #(#query_fns)*
                  }
                },
            ]
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    vec![items, querier].concat()
}

/// This is a hack to fix a clashing name in the stake_authorization module
pub fn fix_clashing_stake_authorization_validators(input: ItemMod) -> ItemMod {
    // do this only if the module is named "stake_authorization"
    if input.ident != "stake_authorization" {
        return input;
    }
    let new_name = Ident::new("Validators_", input.ident.span());
    let mut validators = None;
    let items = input.content.clone().unwrap().1;

    // Iterate over the items in the module and look for the Validators struct then rename it
    let items = items.into_iter().map(|mut item| {
        if let Item::Struct(ref mut s) = item {
            if s.ident == "Validators" {
                s.ident = new_name.clone();
                validators = Some(s.clone());
            }
        }
        item
    });

    // Update any references to the struct
    let items = items.into_iter().map(|mut item| {
        if let Item::Enum(ref mut e) = item {
            if e.ident == "Validators" {
                for v in e.variants.iter_mut() {
                    if let Fields::Unnamed(ref mut f) = v.fields {
                        if let Type::Path(ref mut p) = f.unnamed.first_mut().unwrap().ty {
                            if p.path.segments.first().unwrap().ident == "Validators" {
                                p.path.segments.first_mut().unwrap().ident = new_name.clone();
                            }
                        }
                    }
                }
            }
        }
        item
    });

    ItemMod {
        content: Some((input.content.unwrap().0, items.collect())),
        ..input
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::ItemStruct;

    macro_rules! assert_ast_eq {
        ($left:ident, $right:ident) => {
            let left_fmt =
                prettyplease::unparse(&syn::parse_file(&quote! { #$left }.to_string()).unwrap());
            let right_fmt =
                prettyplease::unparse(&syn::parse_file(&quote! { #$right}.to_string()).unwrap());

            assert!(
                $left == $right,
                "Left is: \n\n{} \n\n but right is: \n\n{} \n\n",
                left_fmt,
                right_fmt
            );
        };
    }

    #[test]
    fn test_add_derive_eq_if_there_is_partial_eq() {
        let item_struct: ItemStruct = syn::parse_quote! {
            #[derive(PartialEq, Debug)]
            struct Hello {
                name: String
            }
        };

        let result = add_derive_eq_struct(&item_struct);
        let expected: ItemStruct = syn::parse_quote! {
            #[derive(PartialEq, Eq, Debug)]
            struct Hello {
                name: String
            }
        };

        assert_ast_eq!(result, expected);
    }

    #[test]
    fn test_add_derive_eq_does_not_add_if_there_is_no_partial_eq() {
        let item_struct: ItemStruct = syn::parse_quote! {
            #[derive(Debug)]
            struct Hello {
                name: String
            }
        };

        let result = add_derive_eq_struct(&item_struct);

        assert_ast_eq!(item_struct, result);
    }

    #[test]
    fn test_add_derive_eq_does_not_add_if_there_is_partial_eq_and_eq() {
        let item_struct: ItemStruct = syn::parse_quote! {
            #[derive(PartialEq, Eq, Debug)]
            struct Hello {
                name: String
            }
        };

        let result = add_derive_eq_struct(&item_struct);

        let expected: ItemStruct = syn::parse_quote! {
            #[derive(PartialEq, Eq, Debug)]
            struct Hello {
                name: String
            }
        };

        assert_ast_eq!(result, expected);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_alias_id_with_ID_if_there_id_a_field_named_id() {
        let item_struct: ItemStruct = syn::parse_quote! {
            #[derive(PartialEq, Eq, Debug)]
            struct PeriodLock {
                id: u64,
                duration: Duration,
            }
        };

        let result = serde_alias_id_with_uppercased(item_struct);

        let expected: ItemStruct = syn::parse_quote! {
            #[derive(PartialEq, Eq, Debug)]
            struct PeriodLock {
                #[serde(alias = "ID")]
                id: u64,
                duration: Duration,
            }
        };

        assert_ast_eq!(result, expected);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_alias_partial_id_with_ID() {
        let item_struct: ItemStruct = syn::parse_quote! {
            #[derive(PartialEq, Eq, Debug)]
            pub struct FeeToken {
                pub denom: ::prost::alloc::string::String,

                pub pool_id: u64,
            }
        };

        let result = serde_alias_id_with_uppercased(item_struct);

        let expected: ItemStruct = syn::parse_quote! {
            #[derive(PartialEq, Eq, Debug)]
            pub struct FeeToken {
                pub denom: ::prost::alloc::string::String,
                #[serde(alias = "poolID")]
                pub pool_id: u64,
            }
        };

        assert_ast_eq!(result, expected);
    }
}
