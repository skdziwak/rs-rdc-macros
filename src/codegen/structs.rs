use proc_macro2::TokenStream;
use quote::{quote};
use syn::{DataStruct, DeriveInput, Field, TypePath, Type, Fields};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use crate::targets::java::implement_java_types;
use crate::utils::find_serde_rename;

pub fn generate_struct_code(input: &DeriveInput, data: &DataStruct) -> TokenStream {
    let name = &input.ident;
    let generics = &input.generics;
    let where_clause = &generics.where_clause;
    let fields = match &data.fields {
        Fields::Named(fields) => &fields.named,
        Fields::Unnamed(_) => panic!("Unnamed fields are not supported"),
        Fields::Unit => panic!("Unit structs are not supported"),
    };
    let field_code = generate_fields_code(fields);

    let java_implements = implement_java_types(name, generics, where_clause);

    quote!(
        impl #generics rdc::codegen::GenerateIR for #name #generics #where_clause {
            fn add_to_ir(ir: &mut rdc::ir::IntermediateRepresentation) {
                let custom_type = ir.target().resolve_custom_type::<#name #generics>();
                let type_name = custom_type.type_name();
                let mut struct_ir = rdc::ir::Struct::new(
                    rdc::ir::Name::from_pascal_case(type_name),
                    custom_type,
                );
                #field_code
                ir.add_struct(struct_ir);
            }
        }

        impl #generics rdc::RDCType for #name #generics #where_clause {}

        #java_implements
    )
}

fn get_json_field_names(fields: Vec<&Field>) -> Vec<String> {
    fields.into_iter().map(|field: &Field| {
        let field_name = field.ident.as_ref().unwrap();
        let serde_rename = find_serde_rename(field.attrs.iter());
        serde_rename.unwrap_or_else(|| field_name.to_string())
    }).collect()
}

fn generate_fields_code(fields: &Punctuated<Field, Comma>) -> TokenStream {
    let field_names: Vec<String> = fields.iter().map(|f| f.ident.as_ref().unwrap().to_string()).collect();
    let field_types: Vec<TypePath> = fields.iter().map(|f| f.ty.clone())
        .map(|t| match t {
            Type::Path(tp) => tp,
            _ => panic!("Unsupported type"),
        })
        .collect();
    let resolved_types: Vec<TokenStream> = field_types.iter().map(|t: &TypePath| {
        quote!(rdc::ir::TypeTarget::Java.resolve_type::<#t>())
    }).collect();
    let dependencies: Vec<TokenStream> = field_types.iter().map(|t: &TypePath| {
        quote!(ir.add::<#t>())
    }).collect();
    let json_field_names = get_json_field_names(fields.iter().collect());
    quote!(
        #({
            let resolved_type = #resolved_types;
            let field = rdc::ir::Field::new(
                rdc::ir::Name::from_snake_case(#field_names),
                #json_field_names,
                resolved_type,
            );
            struct_ir.add_field(field);
            #dependencies
        })*
    )
}
