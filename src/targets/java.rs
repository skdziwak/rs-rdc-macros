use proc_macro2::{Ident, TokenStream};
use quote::{quote};
use syn::{Generics, WhereClause, GenericParam};

pub fn implement_java_types(name: &Ident, generics: &Generics, where_clause: &Option<WhereClause>) -> TokenStream {
    let type_name = match generics.params.len() {
        0 => quote!(
            stringify!(#name).to_string()
        ),
        _ => {
            let generic_types = generics.params.iter().map(|param: &GenericParam| {
                quote!(rdc::ir::TypeTarget::Java.resolve_type::<#param>().type_name())
            });
            quote!({
                let mut type_string = stringify!(#name).to_string();
                #(type_string.push_str(#generic_types);)*
                type_string
            })
        }
    };
    quote!(
        impl #generics rdc::targets::java::type_resolver::JavaCustomType for #name #generics #where_clause {
            fn java_custom_type() -> rdc::ir::CustomType {
                rdc::ir::CustomType::new(#type_name)
            }
        }

        impl #generics rdc::targets::java::type_resolver::JavaType for #name #generics #where_clause {
            fn java_type() -> rdc::ir::Type {
                rdc::ir::Type::new(#type_name)
            }
        }

    )
}
