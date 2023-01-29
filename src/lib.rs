use proc_macro::TokenStream;
use syn::{DeriveInput, Variant};
use crate::codegen::data_enums::generate_data_enum_code;
use crate::codegen::structs::generate_struct_code;
use crate::codegen::unit_enums::generate_unit_enum_code;

mod utils;
mod targets;
mod codegen;

fn generate_code(input: &DeriveInput) -> proc_macro2::TokenStream {
    match &input.data {
        syn::Data::Struct(data) => {
            generate_struct_code(input, data)
        },
        syn::Data::Enum(enum_data) => {
            if enum_data.variants.iter().all(|variant: &Variant| variant.fields.len() == 0) {
                generate_unit_enum_code(input, enum_data)
            } else {
                generate_data_enum_code(input, enum_data)
            }
        },
        syn::Data::Union(_) => panic!("Unions are not supported"),
    }
}

/// Derive macro for generating code for the `rdc` crate.
/// Supported types are structs, enums and primitive types.
#[proc_macro_derive(RDC, attributes(serde))]
pub fn derive_rdc(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
     generate_code(&input).into()
}