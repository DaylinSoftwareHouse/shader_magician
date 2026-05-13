use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn shader(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    // get environment variable settings
    let dbg_tree = std::env::var("MAGICIAN_DBG_TREE")
        .map(|a| a.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    // read manifest
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let src_root = std::path::Path::new(&manifest_dir).join("src");

    // build shader mini-AST
    let idx = magician_rust::index::ProjectIndex::build(&src_root);
    let resolved = magician_rust::resolver::resolve(&input_fn, &idx);
    let tree = magician_rust::stitch::ExtractedTree::from_resolved(resolved);

    // setup for final build
    let tree_src = tree.to_string_pretty();
    let fn_name = &input_fn.sig.ident;
    let const_name = quote::format_ident!("__{}_EXTRACTED_TREE", fn_name.to_string().to_uppercase());

    // debug tree source
    if dbg_tree {
        std::fs::create_dir_all("./shader_out").expect("Failed to create shader_out directory");
        std::fs::write(
            format!("./shader_out/{}.rs", fn_name.to_string().as_str()), 
            &tree_src
        ).expect("Failed to write test output");
    }

    let expanded = quote! {
        #input_fn

        #[doc(hidden)]
        #[allow(non_upper_case_globals, dead_code)]
        pub const #const_name: &str = #tree_src;
    };

    expanded.into()
}