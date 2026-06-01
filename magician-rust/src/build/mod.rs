use std::path::PathBuf;

use crate::*;

pub mod index;
pub mod resolver;
pub mod stitch;
pub mod visit;

pub use index::*;
pub use resolver::*;
pub use stitch::*;
pub use visit::*;

pub fn build(
    src_root: PathBuf,
    out_path: PathBuf,
    dbg_path: Option<PathBuf>
) {
    // setup rerun
    println!("cargo:rerun-if-changed={}", src_root.display());

    // read project and targets
    let index = index::ProjectIndex::build(&src_root);
    let targets = read_targets(&src_root);

    // create folders in output and debug paths
    std::fs::create_dir_all(&out_path).expect("Failed to create folders in output path");
    if let Some(dbg_path) = &dbg_path {
        std::fs::create_dir_all(&dbg_path).expect("Failed to create folders in debug path");
    }

    // for each target function, build shader
    for fn_name in &targets {
        // ensure only operating on functions
        let Some(syn::Item::Fn(entry_fn)) = index.items.get(fn_name) 
            else { continue };

        // build syntax tree for function
        let resolved = resolver::resolve(fn_name, entry_fn, &index);
        let tree = stitch::ExtractedTree::from_resolved(resolved);

        let fn_name = fn_name.split("::").last().unwrap();

        // debug syntax tree if user asks for it
        if let Some(dbg_path) = &dbg_path {
            let mut a = dbg_path.clone();
            a.push(format!("extracted_{}.rs", fn_name));
            std::fs::write(&a, tree.to_string_pretty()).unwrap();

            let mut b = dbg_path.clone();
            b.push(format!("extracted_{}.syntree", fn_name));
            std::fs::write(&b, format!("{:#?}", tree.file)).unwrap();
        
            let raw_translation = Transpiler::new(tree.file.clone());
            let raw_wgsl = raw_translation.transpile_raw();
            let mut c = dbg_path.clone();
            c.push(format!("raw_wgsl_{}.wgsl", fn_name));
            std::fs::write(&c, raw_wgsl).unwrap();
        }

        // execute transcompilation
        let transpiler = Transpiler::new(tree.file);
        let wgsl = transpiler.transpile_wgsl()
            .expect("Failed to transpile wgsl");

        // save final shader
        let mut final_shader_path = out_path.clone();
        final_shader_path.push(format!("{}.wgsl", fn_name));
        std::fs::write(&final_shader_path, wgsl).unwrap();
    }
}

fn read_targets(src_root: &std::path::Path) -> Vec<String> {
    let mut targets = Vec::new();
    collect_shader_fns(src_root, src_root, &mut targets);
    targets
}

fn collect_shader_fns(dir: &std::path::Path, root_dir: &std::path::Path, targets: &mut Vec<String>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("cargo:warning=Could not read dir {}: {}", dir.display(), err);
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_shader_fns(&path, root_dir, targets);
        } else if path.extension().map_or(false, |e| e == "rs") {
            collect_shader_fns_in_file(&path, root_dir, targets);
        }
    }
}

fn collect_shader_fns_in_file(path: &std::path::Path, root_dir: &std::path::Path, targets: &mut Vec<String>) {
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("cargo:warning=Could not read {}: {}", path.display(), err);
            return;
        }
    };

    let file = match syn::parse_file(&src) {
        Ok(f) => f,
        Err(err) => {
            eprintln!("cargo:warning=Could not parse {}: {}", path.display(), err);
            return;
        }
    };

    for item in &file.items {
        if let syn::Item::Fn(f) = item {
            if has_shader_attr(f) {
                targets.push(f.sig.ident.to_string());
                targets.push(index::extract_path(&path.into(), root_dir, &f.sig.ident))
            }
        }
    }
}

fn has_shader_attr(f: &syn::ItemFn) -> bool {
    f.attrs.iter().any(|attr| {
        attr.path().segments.last().map_or(false, |seg| seg.ident == "shader")
    })
}