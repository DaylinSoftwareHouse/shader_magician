use std::path::PathBuf;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, ItemFn, LitStr, Token, parse::{Parse, ParseStream}, parse_macro_input};

#[allow(dead_code)]
struct ShaderArgs {
    path: LitStr,
    stage: Ident,
}

impl Parse for ShaderArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let stage: Ident = input.parse()?;
        Ok(ShaderArgs { path, stage })
    }
}

#[proc_macro_attribute]
pub fn group(_attr: TokenStream, item: TokenStream) -> TokenStream {
    return item;
}

#[proc_macro_attribute]
pub fn shader(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn = item.clone();
    let mut item_fn = parse_macro_input!(item_fn as ItemFn);
    let shader_args = parse_macro_input!(attr as ShaderArgs);
    let shader_out_path = shader_args.path.value();

    // grab shader folder and make sure it exists
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set");
    let mut shader_folder: PathBuf = manifest_dir.into();
    shader_folder.push(shader_out_path);
    std::fs::create_dir_all(&shader_folder).expect("Failed to create shader_out folder");

    // find wgsl file and link it or create compile error
    let mut shader_file = shader_folder.clone();
    shader_file.push(format!("{}.wgsl", item_fn.sig.ident.to_string()));
    let Ok(shader_content) = std::fs::read_to_string(&shader_file)
        else { return item };
    let is_err = shader_content.starts_with("ERROR\n");
    let constant = 
        if is_err { 
            let error = shader_content.clone().split_off(6);
            let error = LitStr::new(&error, Span::call_site());
            quote! { compile_error!(#error) } 
        } else { 
            let path = shader_file.to_str().unwrap();
            let path = LitStr::new(path, Span::call_site());
            quote! { include_str!(#path) } 
        };

    // filter out attributes
    item_fn.sig.inputs.iter_mut().for_each(|input| {
        match input {
            syn::FnArg::Receiver(_receiver) => {},
            syn::FnArg::Typed(pat_type) => {
                pat_type.attrs.retain(|a| {
                    let result = match &a.meta {
                        syn::Meta::Path(_path) => true,
                        syn::Meta::List(_meta_list) => true,
                        syn::Meta::NameValue(meta_name_value) => meta_name_value.path.segments
                            .last().map(|a| {a.ident != "group"}).unwrap_or(true),
                    };

                    if result { panic!("Keeping {a:?}") }

                    result
                });
            }
        }
    });

    // build ident
    let ident = Ident::new(&format!("SHADER_{}", item_fn.sig.ident.to_string()), Span::call_site());

    let expanded = quote! {
        pub const #ident: &str = #constant;

        #item_fn
    };

    expanded.into()
}

#[proc_macro_derive(ShaderLayout, attributes(location, builtin))]
pub fn shader_layout(input: TokenStream) -> TokenStream {
    // decode input
    let input = parse_macro_input!(input as DeriveInput);
    let _struct_name = &input.ident;

    // get only named fields, otherwise, panic
    let _fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("ShaderLayout only supports named fields")
        },
        _ => panic!("ShaderLayout only supports structs")
    };

    TokenStream::new()
}

#[proc_macro_derive(ShaderGroup, attributes(uniform, read, write))]
pub fn shader_group(input: TokenStream) -> TokenStream {
    // decode input
    let input = parse_macro_input!(input as DeriveInput);
    let _struct_name = &input.ident;

    // get only named fields, otherwise, panic
    let _fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("ShaderLayout only supports named fields")
        },
        _ => panic!("ShaderLayout only supports structs")
    };

    TokenStream::new()
}
