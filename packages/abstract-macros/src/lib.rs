extern crate proc_macro2;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::{AttributeArgs, Fields, Item};

#[proc_macro_attribute]
pub fn abstract_response(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(input as syn::Item);
    let attributes = parse_macro_input!(attrs as AttributeArgs);

    let Item::Struct(boot_struct) = &mut item else {
        panic!("Only works on structs");
    };
    let Fields::Unit = &mut boot_struct.fields else {
        panic!("Struct must be unit-struct");
    };
    let name = boot_struct.ident.clone();

    let contract_name = attributes[0].clone();

    let struct_def = quote!(
        struct #name;
        impl #name {
            fn new<T: Into<String>, A: Into<cosmwasm_std::Attribute>>(
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
            fn action<T: Into<String>>(action: T) -> cosmwasm_std::Response {
                #name::new(action, Vec::<cosmwasm_std::Attribute>::new())
            }
        }
    );

    struct_def.into()
}
