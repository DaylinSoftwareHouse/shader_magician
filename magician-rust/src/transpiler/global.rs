use std::sync::atomic::{AtomicU32, Ordering};

use ahash::AHashMap;
use magician_ast::*;

use crate::{TranspiledStruct, TranspiledStructField, Transpiler, expr, stmts};

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
    let ret_ty = match &item.sig.output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => Some(convert_ty(&*ty))
    };
    let block = match item.block.stmts.split_last() {
        // a trailing expression is the function's return value
        Some((syn::Stmt::Expr(tail, None), body)) if ret_ty.is_some() && is_value_expr(tail) => {
            let value = expr::convert_expr(transpiler, &func, tail)
                .expect("Failed to convert the function's trailing return expression");
            let mut stmts = body.iter()
                .flat_map(|a| stmts::convert_stmt(transpiler, &func, a))
                .collect::<Vec<_>>();
            stmts.push(Statement::Return(Some(value)));
            Block { stmts }
        },
        _ => convert_block(transpiler, &func, &*item.block)
    };

    if name == *transpiler.entry_point { attrs.extend(stage_attrs(&transpiler.shader_ty)); }

    (
        name.clone(),
        ShaderElement::Function { attrs, name, params, ret_ty, block, preprocessor_instructions: vec![] }
    )
}

/// Entry point attributes for a stage string from `#[shader(..)]`: `fragment` -> `@fragment`,
/// `compute(8, 8)` -> `@compute @workgroup_size(8, 8)`.
fn stage_attrs(shader_ty: &str) -> Vec<Attr> {
    match shader_ty.split_once('(') {
        Some((stage, dims)) => vec![
            Attr { name: stage.trim().to_string(), content: "".into() },
            Attr { name: "workgroup_size".into(), content: dims.trim_end_matches(')').trim().to_string() },
        ],
        None if shader_ty == "compute" => panic!("Compute shaders need a workgroup size, i.e. `compute(64)`"),
        None => vec![Attr { name: shader_ty.to_string(), content: "".into() }],
    }
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
    let name = item.ident
        .as_ref()
        .expect("Fields must be named for parameters")
        .to_string();

    let ty = convert_ty(&item.ty);

    // address space and access mode from the field's attributes
    let field_attrs = item.attrs.iter()
        .filter_map(|attr| match &attr.meta {
            syn::Meta::Path(path) => path.get_ident().map(|a| a.to_string()),
            _ => None
        })
        .collect::<Vec<_>>();
    let has = |name: &str| field_attrs.iter().any(|a| a == name);
    let declaration =
        if has("workgroup") { Some("workgroup") }
        else if has("uniform") { Some("uniform") }
        else if has("write") || has("read_write") { Some("storage, read_write") }
        else if has("read") || has("storage") { Some("storage, read") }
        else { None };
    let declared_as = declaration.map(|d| format!("var<{d}>")).unwrap_or("var".to_string());

    // workgroup memory is shared by the workgroup rather than bound, so takes no binding slot
    let attrs = if has("workgroup") { vec![] } else {
        let binding = binding_counter.fetch_add(1, Ordering::AcqRel);
        vec![
            Attr { name: "group".to_string(), content: group.to_string() },
            Attr { name: "binding".to_string(), content: binding.to_string() }
        ]
    };

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

/// Returns true if `expr` produces a value as a trailing expression, rather than being
/// control flow that is converted as a statement (`if`, `match`, loops, `return`, ...).
fn is_value_expr(expr: &syn::Expr) -> bool {
    !matches!(expr,
        syn::Expr::If(_) | syn::Expr::Match(_) | syn::Expr::While(_) | syn::Expr::Loop(_) |
        syn::Expr::ForLoop(_) | syn::Expr::Block(_) | syn::Expr::Return(_) | syn::Expr::Break(_) |
        syn::Expr::Continue(_) | syn::Expr::Assign(_)
    )
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
            let segment = type_path.path.segments.last().expect("Unnamed param path");
            let ident = segment.ident.to_string();

            if ident == "BindlessArray" || ident == "RuntimeArray" {
                let inner_ty = match &segment.arguments {
                    syn::PathArguments::AngleBracketed(args) => args.args.iter()
                        .find_map(|a| match a {
                            syn::GenericArgument::Type(t) => Some(t),
                            _ => None
                        })
                        .unwrap_or_else(|| panic!("{ident}<T> requires a type argument")),
                    _ => panic!("{ident}<T> requires a type argument")
                };
                let wgsl_ty = if ident == "BindlessArray" { "binding_array" } else { "array" };
                return format!("{wgsl_ty}<{}>", convert_ty(inner_ty));
            }

            translate_ty_name(&ident).to_string()
        },

        syn::Type::Array(type_array) => {
            // panic!("Type array expr {:?}", type_array.len);
            let size = match &type_array.len {
                syn::Expr::Lit(lit) => match &lit.lit {
                    syn::Lit::Int(lit) => lit.base10_digits().to_string(),
                    _ => panic!("Non-numeric array size found, found {:?}", lit.lit)
                },
                _ => panic!("Non-literal array size found, found {:?}", type_array.len)
            };

            format!("array<{}, {}>", convert_ty(&*type_array.elem), size)
        },

        syn::Type::BareFn(_type_bare_fn) => todo!("Bare function types are not yet supported"),
        syn::Type::Group(_type_group) => todo!("Grouped types are not yet supported"),
        syn::Type::ImplTrait(_type_impl_trait) => todo!("impl Trait types are not yet supported"),
        syn::Type::Infer(_type_infer) => todo!("Infer types are not yet supported"),
        syn::Type::Macro(_type_macro) => todo!("Macro types are not yet supported"),
        syn::Type::Never(_type_never) => todo!("Never types are not yet supported"),
        syn::Type::Paren(_type_paren) => todo!("Parenthesized types are not yet supported"),
        syn::Type::Ptr(_type_ptr) => todo!("Pointer types are not yet supported"),
        syn::Type::Reference(_type_reference) => todo!("Reference types are not yet supported"),
        syn::Type::Slice(_type_slice) => todo!("Slice types are not yet supported"),
        syn::Type::TraitObject(_type_trait_object) => todo!("Trait object types are not yet supported"),
        syn::Type::Tuple(_type_tuple) => todo!("Tuple types are not yet supported"),
        syn::Type::Verbatim(_token_stream) => todo!("Verbatim types are not yet supported"),
        _ => todo!("Unsupported type kind"),
    }
}

pub(crate) fn translate_ty_name(name: &str) -> &str {
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
