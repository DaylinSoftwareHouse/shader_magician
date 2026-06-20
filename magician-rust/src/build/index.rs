use std::{collections::HashMap, fs, path::{Path, PathBuf}};
use syn::{Ident, Item};

/// A flat map of every named item across all .rs files in the project.
/// impl blocks are stored separately because they have no single ident.
#[derive(Default)]
pub struct ProjectIndex {
    pub items: HashMap<String, Item>,
    pub impls: HashMap<String, Vec<Item>>,
    pub uses: HashMap<String, String> // name -> path
}

impl ProjectIndex {
    pub fn build(src_root: &Path) -> Self {
        let mut idx = Self::default();
        walk(&mut idx, src_root, src_root);
        idx
    }

    fn insert_item(&mut self, name: String, item: Item) {
        self.items.insert(name, item);
    }
}

fn walk(idx: &mut ProjectIndex, dir: &Path, root_dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(idx, &path, root_dir);
        } else if path.extension().map_or(false, |e| e == "rs") {
            index_file(idx, &path, root_dir);
        }
    }
}

fn index_file(idx: &mut ProjectIndex, path: &PathBuf, root_dir: &Path) {
    let Ok(src) = fs::read_to_string(path) else { return };
    let Ok(file) = syn::parse_file(&src) else { return };
    index_items(idx, file.items, path, root_dir);
}

fn index_items(idx: &mut ProjectIndex, items: Vec<Item>, file_path: &PathBuf, root_dir: &Path) {
    for item in items {
        match &item {
            Item::Use(u) => { 
                fn recr_tree(idx: &mut ProjectIndex, item: syn::UseTree, segs: &[String]) {
                    match &item {
                        syn::UseTree::Path(use_path) => {
                            let name = use_path.ident.to_string();
                            if name.starts_with("magician_") { return }
                            let mut segs = segs.to_vec();
                            segs.push(name);
                            recr_tree(idx, *use_path.tree.clone(), &segs);
                        },
                        syn::UseTree::Group(use_group) => {
                            for item in &use_group.items {
                                recr_tree(idx, item.clone(), segs);
                            }
                        },
                        syn::UseTree::Name(use_name) => {
                            let mut segs = segs.to_vec();
                            segs.push(use_name.ident.to_string());
                            idx.uses.insert(use_name.ident.to_string(), segs.join("::"));
                        },
                        syn::UseTree::Rename(_use_rename) => todo!("Use rename support"),
                        syn::UseTree::Glob(_use_glob) => {}, // todo!("Use glob support"),
                    }
                }

                recr_tree(idx, u.tree.clone(), &[]);
            }
            Item::Fn(f)     => { idx.insert_item(extract_path(file_path, root_dir, &f.sig.ident), item); }
            Item::Struct(s) => { idx.insert_item(extract_path(file_path, root_dir, &s.ident), item); }
            Item::Enum(e)   => { idx.insert_item(extract_path(file_path, root_dir, &e.ident), item); }
            Item::Trait(t)  => { idx.insert_item(extract_path(file_path, root_dir, &t.ident), item); }
            Item::Type(t)   => { idx.insert_item(extract_path(file_path, root_dir, &t.ident), item); }
            Item::Const(c)  => { idx.insert_item(extract_path(file_path, root_dir, &c.ident), item); }
            Item::Static(s) => { idx.insert_item(extract_path(file_path, root_dir, &s.ident), item); }
            Item::Macro(m)  => {
                if let Some(id) = &m.ident {
                    idx.insert_item(extract_path(file_path, root_dir, &id), item);
                }
            }
            // impl blocks: bucket by the Self type name
            Item::Impl(i) => {
                let type_name = extract_impl_type_name(i, file_path, root_dir);
                idx.impls.entry(type_name).or_default().push(item);
            }
            // Recurse into inline modules
            Item::Mod(m) => {
                if let Some((_, items)) = &m.content {
                    index_items(idx, items.clone(), file_path, root_dir);
                }
            }
            _ => {}
        }
    }
}

fn extract_impl_type_name(i: &syn::ItemImpl, file_path: &PathBuf, root_dir: &Path) -> String {
    // Strip references/boxes to get the base type name
    match i.self_ty.as_ref() {
        syn::Type::Path(tp) => tp.path.segments.last()
            .map(|s| extract_path(file_path, root_dir, &s.ident))
            .unwrap_or_default(),
        _ => String::new(),
    }
}

pub fn extract_path(
    file_path: &PathBuf, 
    root_dir: &Path, 
    ident: &Ident
) -> String {

    // Strip the root dir prefix to get the relative path
    let relative = file_path
        .strip_prefix(root_dir)
        .expect("file_path must be inside root_dir");

    // Remove the .rs extension
    let without_ext = relative.with_extension("");

    // Convert path components to module segments
    let segments: Vec<String> = without_ext
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();

    // Handle lib.rs / main.rs → these ARE the crate root, so just "crate"
    // Handle mod.rs → represents the parent module, drop the "mod" segment
    let segments: Vec<String> = segments
        .into_iter()
        .filter_map(|seg| {
            if seg == "lib" || seg == "main" {
                None // crate root file, contributes no segment
            } else if seg == "mod" {
                None // mod.rs represents its parent dir, drop it
            } else {
                Some(seg)
            }
        })
        .collect();
    
    if segments.is_empty() {
        format!("crate::{}", ident.to_string())
    } else {
        format!("crate::{}::{}", segments.join("::"), ident.to_string())
    }
}
