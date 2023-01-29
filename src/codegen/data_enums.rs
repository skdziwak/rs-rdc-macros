use proc_macro2::{TokenStream};
use quote::{quote};
use syn::{DeriveInput, DataEnum, Variant, Field, Fields, FieldsUnnamed, FieldsNamed};
use crate::targets::java::implement_java_types;
use crate::utils::find_serde_rename;

pub fn generate_data_enum_code(input: &DeriveInput, data: &DataEnum) -> TokenStream {
    let name = &input.ident;
    let generics = &input.generics;
    let where_clause = &generics.where_clause;
    let variants = data.variants.iter().collect::<Vec<_>>();
    let variants_code = variants.iter().map(|v| variant_code(v)).collect::<Vec<_>>();

    let java_implements = implement_java_types(name, generics, where_clause);
    quote!(
        impl #generics rdc::codegen::GenerateIR for #name #generics #where_clause {
            fn add_to_ir(ir: &mut rdc::ir::IntermediateRepresentation) {
                let custom_type = ir.target().resolve_custom_type::<#name #generics>();
                let type_name = custom_type.type_name();
                let mut enum_ir = rdc::ir::DataEnum::new(
                    rdc::ir::Name::from_pascal_case(type_name),
                    custom_type,
                    rdc::ir::DataEnumStyle::External,
                );
                #(#variants_code)*

                ir.add_data_enum(enum_ir);
            }
        }

        impl #generics rdc::RDCType for #name #generics #where_clause {}

        #java_implements
    )
}

fn variant_code(variant: &Variant) -> TokenStream {
    let variant_name = variant.ident.to_string();
    let json_name = find_serde_rename(variant.attrs.iter())
        .unwrap_or_else(|| variant_name.to_string());
    match variant.fields {
        Fields::Unnamed(ref fields) => tuple_variant_code(&variant_name, &json_name, fields),
        Fields::Named(ref fields) => object_variant_code(&variant_name, &json_name, fields),
        Fields::Unit => unit_variant_code(&variant_name, &json_name),
    }
}

fn unit_variant_code(variant_name: &str, json_name: &str) -> TokenStream {
    quote!(
        enum_ir.add_variant(
            rdc::ir::DataEnumVariant::unit(
                rdc::ir::Name::from_pascal_case(#variant_name),
                #json_name.to_string(),
            )
        );
    )
}

fn tuple_variant_code(variant_name: &str, json_name: &str, fields: &FieldsUnnamed) -> TokenStream {
    let field_types = fields.unnamed.iter().map(|f| &f.ty)
        .map(|t| quote!(rdc::ir::TypeTarget::Java.resolve_type::<#t>()));
    let dependencies = fields.unnamed.iter().map(|f| &f.ty)
        .map(|t| quote!(ir.add::<#t>();));
    quote!(
        enum_ir.add_variant(
            rdc::ir::DataEnumVariant::tuple(
                rdc::ir::Name::from_pascal_case(#variant_name),
                #json_name.to_string(),
                vec![#(#field_types),*],
            )
        );
        #(#dependencies)*
    )
}

fn object_variant_code(variant_name: &str, json_name: &str, fields: &FieldsNamed) -> TokenStream {
    let fields = fields.named.iter()
        .map(|field: &Field| {
            let field_name = field.ident.as_ref().unwrap().to_string();
            let json_name = find_serde_rename(field.attrs.iter())
                .unwrap_or_else(|| field_name.to_string());
            let field_type = &field.ty;
            quote!({
                ir.add::<#field_type>();
                rdc::ir::DataEnumObjectField::new(
                    rdc::ir::Name::from_pascal_case(#field_name),
                    #json_name.to_string(),
                    rdc::ir::TypeTarget::Java.resolve_type::<#field_type>(),
                )
            })
        });
    quote!(
        enum_ir.add_variant(
            rdc::ir::DataEnumVariant::object(
                rdc::ir::Name::from_pascal_case(#variant_name),
                #json_name.to_string(),
                vec![#(#fields),*],
            )
        );
    )
}