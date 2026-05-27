use magician_rust::Transpiler;

fn main() {
    let src_root = std::path::Path::new("src");

    println!("cargo:rerun-if-changed=src");

    let index = magician_rust::index::ProjectIndex::build(src_root);

    let targets = read_targets(src_root);

    for fn_name in &targets {
        if let Some(syn::Item::Fn(entry_fn)) = index.items.get(fn_name) {
            let resolved = magician_rust::resolver::resolve(entry_fn, &index);
            let tree = magician_rust::stitch::ExtractedTree::from_resolved(resolved);

            let out_path = format!("./shader_out/extracted_{}.rs", fn_name);
            std::fs::write(&out_path, tree.to_string_pretty()).unwrap();

            let out2_path = format!("./shader_out/extracted_{fn_name}.syntree");
            std::fs::write(&out2_path, format!("{:#?}", tree.file)).unwrap();

            let transpiler = Transpiler::new(tree.file);
            let wgsl = transpiler.transpile_wgsl()
                .expect("Failed to transpile wgsl");
            let out_path = format!("./shader_out/{}.wgsl", fn_name);
            std::fs::write(&out_path, wgsl).unwrap();
        }
    }
}

fn read_targets(src_root: &std::path::Path) -> Vec<String> {
    let mut targets = Vec::new();
    collect_shader_fns(src_root, &mut targets);
    targets
}

fn collect_shader_fns(dir: &std::path::Path, targets: &mut Vec<String>) {
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
            collect_shader_fns(&path, targets);
        } else if path.extension().map_or(false, |e| e == "rs") {
            collect_shader_fns_in_file(&path, targets);
        }
    }
}

fn collect_shader_fns_in_file(path: &std::path::Path, targets: &mut Vec<String>) {
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
            }
        }
    }
}

fn has_shader_attr(f: &syn::ItemFn) -> bool {
    f.attrs.iter().any(|attr| {
        attr.path().segments.last().map_or(false, |seg| seg.ident == "shader")
    })
}