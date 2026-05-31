use std::{collections::HashMap, fs, path::{Path, PathBuf}};
use syn::Item;

/// A flat map of every named item across all .rs files in the project.
/// impl blocks are stored separately because they have no single ident.
#[derive(Default)]
pub struct ProjectIndex {
    pub items: HashMap<String, Item>,
    /// impl blocks keyed by the type name they implement
    pub impls: HashMap<String, Vec<Item>>,
}

impl ProjectIndex {
    pub fn build(src_root: &Path) -> Self {
        let mut idx = Self::default();
        walk(&mut idx, src_root);
        idx
    }

    fn insert_item(&mut self, name: String, item: Item) {
        self.items.insert(name, item);
    }
}

fn walk(idx: &mut ProjectIndex, dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(idx, &path);
        } else if path.extension().map_or(false, |e| e == "rs") {
            index_file(idx, &path);
        }
    }
}

fn index_file(idx: &mut ProjectIndex, path: &PathBuf) {
    let Ok(src) = fs::read_to_string(path) else { return };
    let Ok(file) = syn::parse_file(&src) else { return };
    index_items(idx, file.items);
}

fn index_items(idx: &mut ProjectIndex, items: Vec<Item>) {
    for item in items {
        match &item {
            Item::Fn(f)     => { idx.insert_item(f.sig.ident.to_string(), item); }
            Item::Struct(s) => { idx.insert_item(s.ident.to_string(), item); }
            Item::Enum(e)   => { idx.insert_item(e.ident.to_string(), item); }
            Item::Trait(t)  => { idx.insert_item(t.ident.to_string(), item); }
            Item::Type(t)   => { idx.insert_item(t.ident.to_string(), item); }
            Item::Const(c)  => { idx.insert_item(c.ident.to_string(), item); }
            Item::Static(s) => { idx.insert_item(s.ident.to_string(), item); }
            Item::Macro(m)  => {
                if let Some(id) = &m.ident {
                    idx.insert_item(id.to_string(), item);
                }
            }
            // impl blocks: bucket by the Self type name
            Item::Impl(i) => {
                let type_name = extract_impl_type_name(i);
                idx.impls.entry(type_name).or_default().push(item);
            }
            // Recurse into inline modules
            Item::Mod(m) => {
                if let Some((_, items)) = &m.content {
                    index_items(idx, items.clone());
                }
            }
            _ => {}
        }
    }
}

fn extract_impl_type_name(i: &syn::ItemImpl) -> String {
    // Strip references/boxes to get the base type name
    match i.self_ty.as_ref() {
        syn::Type::Path(tp) => tp.path.segments.last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default(),
        _ => String::new(),
    }
}
