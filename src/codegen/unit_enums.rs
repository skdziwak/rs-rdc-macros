use proc_macro2::TokenStream;
use quote::{quote};
use syn::{DeriveInput, DataEnum, Variant};
use crate::targets::java::implement_java_types;
use crate::utils::find_serde_rename;

pub fn generate_unit_enum_code(input: &DeriveInput, data: &DataEnum) -> TokenStream {
    let name = &input.ident;
    let generics = &input.generics;
    let where_clause = &generics.where_clause;
    let variants = data.variants.iter().collect::<Vec<_>>();
    let variants_code = variants.iter().map(|v| generate_variant(v)).collect::<Vec<_>>();

    let java_implements = implement_java_types(name, generics, where_clause);
    quote!(
        impl #generics rdc::codegen::GenerateIR for #name #generics #where_clause {
            fn add_to_ir(ir: &mut rdc::ir::IntermediateRepresentation) {
                let custom_type = ir.target().resolve_custom_type::<#name #generics>();
                let type_name = custom_type.type_name();
                let mut enum_ir = rdc::ir::Enum::new(
                    rdc::ir::Name::from_pascal_case(type_name),
                    custom_type,
                );
                #(#variants_code)*

                ir.add_enum(enum_ir);
            }
        }

        impl #generics rdc::RDCType for #name #generics #where_clause {}

        #java_implements
    )
}
fn generate_variant(variant: &Variant) -> TokenStream {
    let variant_name = &variant.ident.to_string();
    if variant.fields.len() != 0 {
        panic!("Unit enums must have no fields");
    }
    let json_name = find_serde_rename(variant.attrs.iter())
        .unwrap_or_else(|| variant_name.to_string());

    quote!({
        enum_ir.add_variant(
            rdc::ir::EnumVariant::new(
                rdc::ir::Name::from_pascal_case(#variant_name),
                #json_name,
            )
        )
    })
}
