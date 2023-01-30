extern crate proc_macro2;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::{AttributeArgs, Fields, Item};

const DELIMITER: &str = ",";

/// Add the abstract event with action to a cosmwasm_std::Response.
#[proc_macro]
pub fn with_abstract_event(input: TokenStream) -> TokenStream {
    let input = input.to_string();
    let mut input = input.split(DELIMITER);
    let base_response = input.next().unwrap().trim();
    let contract_name = input.next().unwrap().trim();
    let action = input.next().unwrap().trim();
    // Collect the remaining
    let attrs = input.collect::<Vec<&str>>().join(DELIMITER);

    let attribute_addition = if attrs.is_empty() {
        "".to_string()
    } else if attrs.starts_with('[') && attrs.ends_with(']') {
        format!(".add_attributes(vec!{attrs})")
    } else {
        format!(".add_attributes({attrs})")
    };
    let output = format!(
        "{base_response}
        .add_event(
            cosmwasm_std::Event::new(\"abstract\")
                .add_attribute(\"contract\", {contract_name})
                .add_attribute(\"action\", {action})
                {attribute_addition}
        )"
    );

    output.parse().unwrap()
}

#[proc_macro_attribute]
pub fn abstract_response(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(input as syn::Item);
    let attributes = parse_macro_input!(attrs as AttributeArgs);

    let Item::Struct(resp_struct) = &mut item else {
        panic!("Only works on structs");
    };
    let Fields::Unit = &mut resp_struct.fields else {
        panic!("Struct must be unit-struct");
    };
    let visibility = resp_struct.vis.clone();
    let resp_name = resp_struct.ident.clone();

    // if attributes.is_empty() {}

    let contract_name = attributes[0].clone();

    let struct_def = quote!(
        #visibility struct #resp_name;
        impl #resp_name {
            #visibility fn new<T: Into<String>, A: Into<cosmwasm_std::Attribute>>(
                action: T,
                attrs: impl IntoIterator<Item = A>,
            ) -> cosmwasm_std::Response {
                cosmwasm_std::Response::new().add_event(
                    cosmwasm_std::Event::new("abstract")
                        .add_attributes(vec![("contract", #contract_name)])
                        .add_attributes(vec![("action", action)])
                        .add_attributes(attrs),
                )
            }
            #visibility fn action<T: Into<String>>(action: T) -> cosmwasm_std::Response {
                #resp_name::new(action, Vec::<cosmwasm_std::Attribute>::new())
            }
        }
    );

    struct_def.into()
}
