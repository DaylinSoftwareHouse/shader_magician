use std::{collections::HashMap, sync::atomic::{AtomicU32, Ordering}};

use magician_ast::*;

pub fn transpile(file: &syn::File) -> String {
    let counter = AtomicU32::new(0);
    let mut globals = Vec::new();
    let mut structs = Vec::new();
    let mut functions = Vec::new();
    let mut global_struct_names = Vec::new();

    file.items.iter().for_each(|item| match &item {
        syn::Item::Struct(item_struct) => {

            let is_group = item_struct.attrs.iter().any(|attr| {
                match &attr.meta {
                    syn::Meta::List(list) => {
                        let derive = list.path.get_ident().map(|a| a.to_string() == "derive").unwrap_or(false);
                        let is_group = list.tokens.clone()
                            .into_iter().next()
                            .map(|ident| {
                                match &ident {
                                    proc_macro2::TokenTree::Ident(ident) => ident.to_string() == "ShaderGroup",
                                    _ => false
                                }
                            })
                            .unwrap_or(false);

                        derive && is_group
                    },
                    _ => false
                }
            });

            if is_group {
                globals.extend(convert_global_struct(item_struct, &counter));
                global_struct_names.push(item_struct.ident.to_string());
            } else { 
                structs.push(convert_non_global_struct(item_struct));
            }
        },
        
        syn::Item::Const(_item_const) => todo!(),
        syn::Item::Enum(_item_enum) => todo!(),
        syn::Item::ExternCrate(_item_extern_crate) => todo!(),
        syn::Item::Fn(item_fn) => { functions.push(convert_function(item_fn, &global_struct_names)); }, //todo!(),
        syn::Item::ForeignMod(_item_foreign_mod) => todo!(),
        syn::Item::Impl(_item_impl) => todo!(),
        syn::Item::Macro(_item_macro) => todo!(),
        syn::Item::Mod(_item_mod) => todo!(),
        syn::Item::Static(_item_static) => todo!(),
        syn::Item::Trait(_item_trait) => todo!(),
        syn::Item::TraitAlias(_item_trait_alias) => todo!(),
        syn::Item::Type(_item_type) => todo!(),
        syn::Item::Union(_item_union) => todo!(),
        syn::Item::Use(_item_use) => panic!("Imports should have been stripped before this stage!"),
        syn::Item::Verbatim(_token_stream) => panic!("Cannot handle verbatim syntax items here!"),
        _ => todo!(),
    });

    let mut output = String::new();
    let replacements = HashMap::new();
    output.push_str(&ShaderElement::to_wgsl(&globals, &replacements, false));
    output.push_str("\n");
    output.push_str(&ShaderElement::to_wgsl(&structs, &replacements, false));
    output.push_str("\n");
    output.push_str(&ShaderElement::to_wgsl(&functions, &replacements, false));
    return output;
}

fn convert_function(item: &syn::ItemFn, struct_global_names: &[String]) -> ShaderElement {
    let attrs = Vec::new();

    let name = item.sig.ident.to_string();
    let block = convert_block(&*item.block);
    let params = item.sig.inputs.iter()
        .filter(|fn_arg| match fn_arg {
            syn::FnArg::Receiver(_) => true,
            syn::FnArg::Typed(ty) => !struct_global_names.contains(&convert_ty(&ty.ty))
        })
        .map(convert_fn_arg).collect::<Vec<_>>();
    let ret_ty = match &item.sig.output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => Some(convert_ty(&*ty))
    };

    ShaderElement::Function { attrs, name, params, ret_ty, block, preprocessor_instructions: vec![] }
}

fn convert_non_global_struct(item: &syn::ItemStruct) -> ShaderElement {
    let name = item.ident.to_string();
    let params = item.fields.iter()
        .map(|field| convert_param(field))
        .collect::<Vec<_>>();

    ShaderElement::Struct { attrs: vec![], name, params }
}

fn convert_global_struct(item: &syn::ItemStruct, counter: &AtomicU32) -> Vec<ShaderElement> {
    item.fields.iter()
        .map(|field| convert_global(field, counter))
        .collect()
}

fn convert_global(item: &syn::Field, counter: &AtomicU32) -> ShaderElement {
    let binding = counter.fetch_add(1, Ordering::AcqRel);
    let attrs = vec![
        Attr { name: "group".to_string(), content: "0".to_string() },
        Attr { name: "binding".to_string(), content: binding.to_string() }
    ];

    let name = item.ident
        .as_ref()
        .expect("Fields must be named for parameters")
        .to_string();

    let ty = convert_ty(&item.ty);

    let mut declaration = Vec::new();
    item.attrs.iter().for_each(|attr| {
        match &attr.meta {
            syn::Meta::Path(path) => {
                let Some(path) = path.get_ident() else { return };
                declaration.push(path.to_string());
            },
            _ => {}
        }
    });
    let declared_as = format!("var<{}>", declaration.join(", "));

    ShaderElement::Global { attrs, declared_as, name, ty }
}

fn convert_fn_arg(item: &syn::FnArg) -> Param {
    match item {
        syn::FnArg::Receiver(receiver) => todo!("Receiver type {receiver:?}"),
        syn::FnArg::Typed(item) => {
            let name = match &*item.pat {
                syn::Pat::Ident(ident) => ident.ident.to_string(),
                _ => todo!("PatType handler for {item:?}")
            };

            let attrs = item.attrs.iter()
                .filter_map(translate_attr)
                .collect::<Vec<_>>();

            let ty = convert_ty(&item.ty);

            Param { attrs, name, ty }
        }
    }
}

fn convert_block(item: &syn::Block) -> Block {
    Block { stmts: item.stmts.iter().map(convert_stmt).collect() }
}

fn convert_param(item: &syn::Field) -> Param {
    let name = item.ident
        .as_ref()
        .expect("Fields must be named for parameters")
        .to_string();

    let attrs = item.attrs.iter()
        .filter_map(translate_attr)
        .collect::<Vec<_>>();

    let ty = convert_ty(&item.ty);

    Param { attrs, name, ty }
}

fn convert_ty(ty: &syn::Type) -> String {
    match &ty {
        syn::Type::Path(type_path) => {
            let ident = type_path.path.segments.last()
                .expect("Unnamed param path")
                .ident
                .to_string();
            translate_ty_name(&ident).to_string()
        },

        syn::Type::Array(_type_array) => todo!(),
        syn::Type::BareFn(_type_bare_fn) => todo!(),
        syn::Type::Group(_type_group) => todo!(),
        syn::Type::ImplTrait(_type_impl_trait) => todo!(),
        syn::Type::Infer(_type_infer) => todo!(),
        syn::Type::Macro(_type_macro) => todo!(),
        syn::Type::Never(_type_never) => todo!(),
        syn::Type::Paren(_type_paren) => todo!(),
        syn::Type::Ptr(_type_ptr) => todo!(),
        syn::Type::Reference(_type_reference) => todo!(),
        syn::Type::Slice(_type_slice) => todo!(),
        syn::Type::TraitObject(_type_trait_object) => todo!(),
        syn::Type::Tuple(_type_tuple) => todo!(),
        syn::Type::Verbatim(_token_stream) => todo!(),
        _ => todo!(),
    }
}

fn convert_stmt(stmt: &syn::Stmt) -> Statement {
    todo!()
}

fn translate_ty_name(name: &str) -> &str {
    match name {
        "Vec2" => "vec2<f32>",
        "Vec3" => "vec3<f32>",
        "Vec4" => "vec4<f32>",
        "DVec2" => "vec2<f64>",
        "DVec3" => "vec3<f64>",
        "DVec4" => "vec4<f64>",
        "IVec2" => "vec2<i32>",
        "IVec3" => "vec3<i32>",
        "IVec4" => "vec4<i32>",
        "UVec2" => "vec2<u32>",
        "UVec3" => "vec3<u32>",
        "UVec4" => "vec4<u32>",
        "BVec2" => "vec2<bool>",
        "BVec3" => "vec3<bool>",
        "BVec4" => "vec4<bool>",
        other => other
    }
}

fn translate_attr(attr: &syn::Attribute) -> Option<Attr> {
    let (attr_name, arguments) = match &attr.meta {
        syn::Meta::Path(path) => {
            let ident = path
                .get_ident()
                .expect("Meta::Path in attribute had no identifier")
                .to_string();
            let value = None;
            (ident, value)
        },
        syn::Meta::List(_meta_list) => todo!("Meta::List attributes"),
        syn::Meta::NameValue(meta_name_value) => {
            let ident = meta_name_value.path
                .get_ident()
                .expect("Meta::NameValue in attribute has no identifier")
                .to_string();
            let value = &meta_name_value.value;
            (ident, Some(value))
        },
    };

    match attr_name.as_str() {
        "uniform" => Some(Attr { name: "uniform".to_string(), content: "".to_string() }), // todo
        "location" => {
            let Some(loc) = arguments else { panic!("Location attribute must have arguments") };
            let syn::Expr::Lit(loc) = loc else { panic!("Location attribute must have literal") };
            let syn::Lit::Int(loc) = &loc.lit else { panic!("Location attribute must have literal integer") };
            let loc = loc.base10_parse::<u32>().expect("Failed to base10 parse location attribute literal integer");
            Some(Attr { name: "location".to_string(), content: loc.to_string() })
        },
        "builtin" => {
            let Some(loc) = arguments else { panic!("Builtin attribute must have arguments") };
            let syn::Expr::Lit(loc) = loc else { panic!("Builtin attribute must have literal") };
            let syn::Lit::Str(loc) = &loc.lit else { panic!("Builtin attribute must have literal string") };
            Some(Attr { name: "builtin".to_string(), content: loc.value() })
        },
        _ => None
    }
}
