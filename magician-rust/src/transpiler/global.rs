use std::sync::atomic::{AtomicU32, Ordering};

use ahash::AHashMap;
use magician_ast::*;

use crate::{TranspiledStruct, TranspiledStructField, Transpiler, stmts};

#[derive(Default)]
pub struct FunctionContext {
    pub global_params: AHashMap<String, String> // param name -> param type
}

pub(crate) fn convert_function(
    transpiler: &mut Transpiler, 
    item: &syn::ItemFn
) -> (String, ShaderElement) {
    let mut attrs = Vec::new();
    let mut func = FunctionContext::default();

    let name = item.sig.ident.to_string();
    let params = item.sig.inputs.iter()
        .filter(|fn_arg| match fn_arg {
            syn::FnArg::Receiver(_) => true,
            syn::FnArg::Typed(pat) => {
                let ty = convert_ty(&pat.ty);
                let is_global = transpiler.unfinished_globals.contains_key(&ty);

                if is_global {
                    let name = match &*pat.pat {
                        syn::Pat::Ident(ident) => ident.ident.to_string(),
                        _ => todo!("PatType handler for {item:?}")
                    };
                    func.global_params.insert(name, ty.clone());

                    let group = pat.attrs.iter()
                        .filter_map(|attr| {
                            match &attr.meta {
                                syn::Meta::Path(_path) => None,
                                syn::Meta::List(_meta_list) => None,
                                syn::Meta::NameValue(meta_name_value) => {
                                    let is_group = meta_name_value.path.segments.last()
                                        .map(|a| a.ident.to_string() == "group")
                                        .unwrap_or(false);
                                    
                                    if is_group {
                                        match &meta_name_value.value {
                                            syn::Expr::Lit(expr_lit) => {
                                                match &expr_lit.lit {
                                                    syn::Lit::Int(lit_int) => lit_int.base10_parse::<u32>().ok(),
                                                    _ => None
                                                }
                                            },
                                            _ => None
                                        }
                                    } else { None }
                                }
                            }
                        })
                        .next();

                    if let Some(group_idx) = group {
                        if let Some(item_struct) = transpiler.unfinished_globals.remove(&ty) {
                            let globals = convert_global_struct(&item_struct, group_idx);
                            globals.into_iter().for_each(|(global_name, global)| {
                                transpiler.globals.insert(global_name, global);
                            });
                        }
                    } 
                }

                !is_global
            }
        })
        .map(convert_fn_arg).collect::<Vec<_>>();
    let block = convert_block(transpiler, &func, &*item.block);
    let ret_ty = match &item.sig.output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => Some(convert_ty(&*ty))
    };

    if name == *transpiler.entry_point { attrs.push(Attr { name: transpiler.shader_ty.clone(), content: "".into() }); }

    (
        name.clone(),
        ShaderElement::Function { attrs, name, params, ret_ty, block, preprocessor_instructions: vec![] }
    )
}

pub(crate) fn convert_non_global_struct(item: &syn::ItemStruct) -> (String, TranspiledStruct) {
    let name = item.ident.to_string();
    let params = item.fields.iter()
        .map(|field| convert_param(field))
        .collect::<Vec<_>>();

    let fields = params.iter()
        .map(|field| {
            TranspiledStructField { name: field.name.clone(), ty: field.ty.clone() }
        })
        .collect::<Vec<_>>();

    (
        name.clone(),
        TranspiledStruct { 
            element: ShaderElement::Struct { attrs: vec![], name, params }, 
            fields
        }
    )
}

pub(crate) fn convert_global_struct(item: &syn::ItemStruct, group: u32) -> Vec<(String, ShaderElement)> {
    let binding_counter = AtomicU32::new(0);
    item.fields.iter()
        .map(|field| convert_global(field, group, &binding_counter))
        .collect()
}

pub(crate) fn convert_global(item: &syn::Field, group: u32, binding_counter: &AtomicU32) -> (String, ShaderElement) {
    let binding = binding_counter.fetch_add(1, Ordering::AcqRel);
    let attrs = vec![
        Attr { name: "group".to_string(), content: group.to_string() },
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
    let declared_as = if declaration.is_empty() { "var".to_string() } else { format!("var<{}>", declaration.join(", ")) };

    (name.clone(), ShaderElement::Global { attrs, declared_as, name, ty })
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

fn convert_block(transpiler: &Transpiler, func: &FunctionContext, item: &syn::Block) -> Block {
    Block { stmts: item.stmts.iter().flat_map(|a| stmts::convert_stmt(transpiler, func, a)).collect() }
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

pub fn convert_ty(ty: &syn::Type) -> String {
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
        "Mat2" => "mat2x2<f32>",
        "Mat3" => "mat3x3<f32>",
        "Mat4" => "mat4x4<f32>",
        "DMat2" => "mat2x2<f64>",
        "DMat3" => "mat3x3<f64>",
        "DMat4" => "mat4x4<f64>",
        "IMat2" => "mat2x2<i32>",
        "IMat3" => "mat3x3<i32>",
        "IMat4" => "mat4x4<i32>",
        "UMat2" => "mat2x2<u32>",
        "UMat3" => "mat3x3<u32>",
        "UMat4" => "mat4x4<u32>",
        "BMat2" => "mat2x2<bool>",
        "BMat3" => "mat3x3<bool>",
        "BMat4" => "mat4x4<bool>",
        "Texture2D" => "texture_2d<f32>",
        "Sampler" => "sampler",
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
