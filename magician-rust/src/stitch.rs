use syn::{File, Item};
use crate::resolver::ResolvedSet;

/// A self-contained syn::File containing the entry function
/// and all its transitive dependencies, in dependency order.
pub struct ExtractedTree {
    pub file: syn::File,
    /// Flat list of item names in order, for debugging
    pub item_names: Vec<String>,
}

impl ExtractedTree {
    pub fn from_resolved(resolved: ResolvedSet) -> Self {
        let item_names = resolved.ordered.iter()
            .map(item_name)
            .collect();

        let file = File {
            shebang: None,
            attrs: vec![],
            items: resolved.ordered,
        };

        Self { file, item_names }
    }

    /// Pretty-print the tree (useful for debugging / saving to disk)
    pub fn to_token_stream(&self) -> proc_macro2::TokenStream {
        use quote::ToTokens;
        self.file.to_token_stream()
    }

    pub fn to_string_pretty(&self) -> String {
        prettyplease::unparse(&self.file)
    }
}

fn item_name(item: &Item) -> String {
    match item {
        Item::Fn(f)     => f.sig.ident.to_string(),
        Item::Struct(s) => s.ident.to_string(),
        Item::Enum(e)   => e.ident.to_string(),
        Item::Trait(t)  => t.ident.to_string(),
        Item::Type(t)   => t.ident.to_string(),
        Item::Const(c)  => c.ident.to_string(),
        Item::Static(s) => s.ident.to_string(),
        Item::Impl(i)   => format!("impl {}", extract_impl_type(i)),
        _               => "<unknown>".into(),
    }
}

fn extract_impl_type(i: &syn::ItemImpl) -> String {
    match i.self_ty.as_ref() {
        syn::Type::Path(tp) => quote::quote!(#tp).to_string(),
        _ => "?".into(),
    }
}
