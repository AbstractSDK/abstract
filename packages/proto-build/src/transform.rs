use heck::ToUpperCamelCase;
use log::debug;
use prost_types::FileDescriptorSet;
use quote::ToTokens;
use regex::Regex;
use std::ffi::OsStr;
use std::fs::{create_dir_all, remove_dir_all};
use std::path::{Path, PathBuf};
use std::{fs, io};
use syn::{parse_quote, File, Item, ItemMod};
use walkdir::WalkDir;

use crate::transformers;

/// Protos belonging to these Protobuf packages will be excluded
/// (i.e. because they are sourced from `tendermint-proto`)
const EXCLUDED_PROTO_PACKAGES: &[&str] = &["cosmos_proto", "gogoproto", "google", "tendermint"];

pub fn copy_and_transform_all(from_dir: &Path, to_dir: &Path, descriptor: &FileDescriptorSet) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let to_dir = root.join(to_dir);
    debug!("Copying generated files into '{}'...", to_dir.display());

    // Remove old compiled files
    remove_dir_all(&to_dir).unwrap_or_default();
    create_dir_all(&to_dir).unwrap();

    let mut filenames = Vec::new();

    // Copy new compiled files (prost does not use folder structures)
    let errors = WalkDir::new(from_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| {
            let filename = e.file_name().to_os_string().to_str().unwrap().to_string();
            filenames.push(filename.clone());
            copy_and_transform(
                e.path(),
                format!("{}/{}", to_dir.display(), &filename),
                descriptor,
            )
        })
        .filter_map(|e| e.err())
        .collect::<Vec<_>>();

    if !errors.is_empty() {
        for e in errors {
            eprintln!("[error] Error while copying compiled file: {}", e);
        }

        panic!("[error] Aborted.");
    }
}

fn copy_and_transform(
    src: &Path,
    dest: impl AsRef<Path>,
    descriptor: &FileDescriptorSet,
) -> io::Result<()> {
    // Skip proto files belonging to `EXCLUDED_PROTO_PACKAGES`
    for package in EXCLUDED_PROTO_PACKAGES {
        if let Some(filename) = src.file_name().and_then(OsStr::to_str) {
            if filename.starts_with(&format!("{}.", package)) {
                return Ok(());
            }
        }
    }

    let mut contents = match fs::read_to_string(src) {
        Ok(c) => c,
        Err(e) => {
            debug!("{:?} – {}, copy_and_transform skipped", src, e);
            return Ok(());
        }
    };

    for &(regex, replacement) in transformers::REPLACEMENTS {
        contents = Regex::new(regex)
            .unwrap_or_else(|_| panic!("invalid regex: {}", regex))
            .replace_all(&contents, replacement)
            .to_string();
    }

    let file = syn::parse_file(&contents);
    if let Ok(file) = file {
        // only transform rust file (skipping `*_COMMIT` file)
        let items = transform_module(file.items, src, &[], descriptor, false);
        contents = prettyplease::unparse(&File { items, ..file });
    }

    fs::write(dest, &*contents)
}

fn transform_module(
    items: Vec<Item>,
    src: &Path,
    ancestors: &[String],
    descriptor: &FileDescriptorSet,
    nested_mod: bool,
) -> Vec<Item> {
    let items = transform_items(items, src, ancestors, descriptor);
    let items = prepend(items);

    append(items, src, descriptor, nested_mod)
}

fn prepend(items: Vec<Item>) -> Vec<Item> {
    let mut items = items;

    let mut prepending_items = vec![syn::parse_quote! {
        use osmosis_std_derive::CosmwasmExt;
    }];

    items.splice(0..0, prepending_items.drain(..));
    items
}

fn append(
    items: Vec<Item>,
    src: &Path,
    descriptor: &FileDescriptorSet,
    nested_mod: bool,
) -> Vec<Item> {
    transformers::append_querier(items, src, nested_mod, descriptor)
}

fn transform_items(
    items: Vec<Item>,
    src: &Path,
    ancestors: &[String],
    descriptor: &FileDescriptorSet,
) -> Vec<Item> {
    // TODO: Remove this temporary hack when cosmos & tendermint code gen is supported
    let remove_struct_fields_that_depends_on_tendermint_proto = |i: Item| match i.clone() {
        Item::Struct(s) => {
            let is_depending_on_tendermint = s.fields.iter().any(|field| {
                let tt = field.ty.to_token_stream();
                tt.to_string().contains("tendermint_proto")
            });

            if is_depending_on_tendermint {
                let ident = s.ident;
                Item::Struct(parse_quote! {
                    /// NOTE: The following type is not implemented due to current limitations of code generator
                    /// which currently has issue with tendermint_proto.
                    /// This will be fixed in the upcoming release.
                    #[allow(dead_code)]
                    struct #ident {}
                })
            } else {
                Item::Struct(s)
            }
        }
        _ => i,
    };
    items
        .into_iter()
        .map(|i| match i {
            Item::Struct(s) => Item::Struct({
                let s = transformers::add_derive_eq_struct(&s);
                let s = transformers::append_attrs_struct(src, &s, descriptor);
                let s = transformers::serde_alias_id_with_uppercased(s);
                transformers::allow_serde_int_as_str(s)
            }),

            Item::Enum(e) => Item::Enum({
                let e = transformers::add_derive_eq_enum(&e);
                transformers::append_attrs_enum(src, &e, descriptor)
            }),

            // This is a temporary hack to fix the issue with clashing stake authorization validators
            Item::Mod(m) => Item::Mod(transformers::fix_clashing_stake_authorization_validators(m)),

            i => i,
        })
        // TODO: Remove this temporary hack when cosmos & tendermint code gen is supported
        .map(remove_struct_fields_that_depends_on_tendermint_proto)
        .map(|i: Item| transform_nested_mod(i, src, ancestors, descriptor))
        .collect::<Vec<Item>>()
}

fn transform_nested_mod(
    i: Item,
    src: &Path,
    ancestors: &[String],
    descriptor: &FileDescriptorSet,
) -> Item {
    match i.clone() {
        Item::Mod(m) => {
            let parent = &m.ident.to_string().to_upper_camel_case();
            let content = m.content.map(|(brace, items)| {
                (
                    brace,
                    transform_module(
                        items,
                        src,
                        &[ancestors, &[parent.to_string()]].concat(),
                        descriptor,
                        true,
                    ),
                )
            });

            Item::Mod(ItemMod { content, ..m })
        }
        _ => i,
    }
}
