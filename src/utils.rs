use syn::Attribute;

pub fn find_serde_rename<'a>(mut attributes: impl Iterator<Item=&'a Attribute>) -> Option<String> {
    attributes.find(|attr| {
        attr.path.is_ident("serde")
    }).and_then(|attr| {
        attr.parse_meta().ok()
    }).and_then(|meta| {
        if let syn::Meta::List(list) = meta {
            list.nested.iter().find(|nested| {
                if let syn::NestedMeta::Meta(meta) = nested {
                    meta.path().is_ident("rename")
                } else {
                    false
                }
            }).and_then(|nested| {
                if let syn::NestedMeta::Meta(meta) = nested {
                    if let syn::Meta::NameValue(name_value) = meta {
                        if let syn::Lit::Str(lit_str) = &name_value.lit {
                            Some(lit_str.value())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        } else {
            None
        }
    })
}