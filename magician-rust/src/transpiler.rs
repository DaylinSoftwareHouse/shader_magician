use std::collections::HashMap;

use magician_ast::*;

pub fn transpile(file: &syn::File) -> String {
    let mut structs = Vec::new();
    let functions = Vec::new();

    file.items.iter().for_each(|item| match &item {
        syn::Item::Struct(item_struct) => { structs.push(convert_struct(item_struct)); },
        
        syn::Item::Const(_item_const) => todo!(),
        syn::Item::Enum(_item_enum) => todo!(),
        syn::Item::ExternCrate(_item_extern_crate) => todo!(),
        syn::Item::Fn(_item_fn) => {}, //todo!(),
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
    output.push_str(&ShaderElement::to_wgsl(&structs, &replacements, false));
    output.push_str(&ShaderElement::to_wgsl(&functions, &replacements, false));
    return output;
}

fn convert_struct(item: &syn::ItemStruct) -> ShaderElement {
    let name = item.ident.to_string();
    let params = item.fields.iter()
        .map(|field| convert_param(field))
        .collect::<Vec<_>>();

    ShaderElement::Struct { attrs: vec![], name, params }
}

fn convert_param(item: &syn::Field) -> Param {
    let name = item.ident
        .as_ref()
        .expect("Fields must be named for parameters")
        .to_string();

    let ty = match &item.ty {
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
    };

    Param { attrs: vec![], name, ty }
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
