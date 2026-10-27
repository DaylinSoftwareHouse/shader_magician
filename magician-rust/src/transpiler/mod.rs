use std::collections::HashMap;

use ahash::AHashMap;
use global::*;
use magician_ast::*;
use naga::valid::{Capabilities, ValidationFlags, Validator};
use syn::ItemStruct;

pub(crate) mod expr;
pub(crate) mod global;
pub(crate) mod stmts;

pub struct Transpiler {
    file: syn::File,
    globals: AHashMap<String, ShaderElement>,
    structs: AHashMap<String, TranspiledStruct>,
    functions: AHashMap<String, ShaderElement>,
    unfinished_globals: HashMap<String, ItemStruct>,
    entry_point: String,
    shader_ty: String
}

pub struct TranspiledStruct {
    pub element: ShaderElement,
    pub fields: Vec<TranspiledStructField>
}

pub struct TranspiledStructField {
    pub name: String,
    pub ty: String
}

impl Transpiler {
    pub fn new(file: syn::File, entry_point: String, shader_ty: String) -> Self {
        Self {
            file,
            globals: AHashMap::new(),
            structs: AHashMap::new(),
            functions: AHashMap::new(),
            unfinished_globals: HashMap::new(),
            entry_point, shader_ty
        }
    }

    /// Returns true if a function named `name` is defined in the file being transpiled.
    pub(crate) fn is_user_fn(&self, name: &str) -> bool {
        self.file.items.iter().any(|item| matches!(item, syn::Item::Fn(f) if f.sig.ident == name))
    }

    pub fn transpile_raw(mut self) -> String {
        for item in self.file.items.clone() {
            match item {
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
                        // let globals = convert_global_struct(&item_struct, &counter);
                        // globals.into_iter().for_each(|(global_name, global)| {
                        //     self.globals.insert(global_name, global);
                        // });
                        self.unfinished_globals.insert(item_struct.ident.to_string(), item_struct.clone());
                    } else { 
                        let (struct_name, s) = convert_non_global_struct(&item_struct);
                        self.structs.insert(struct_name, s);
                    }
                },
                
                syn::Item::Const(_item_const) => todo!("Const items are not yet supported"),
                syn::Item::Enum(_item_enum) => todo!("Enum items are not yet supported"),
                syn::Item::ExternCrate(_item_extern_crate) => todo!("Extern crate items are not yet supported"),
                syn::Item::Fn(item_fn) => { 
                    let function = convert_function(&mut self, &item_fn);
                    self.functions.insert(function.0, function.1);
                },
                syn::Item::ForeignMod(_item_foreign_mod) => todo!("ForeignMod"),
                syn::Item::Impl(_item_impl) => {}, // todo!("Impls"),
                syn::Item::Macro(_item_macro) => todo!("Macros"),
                syn::Item::Mod(_item_mod) => todo!("Mod"),
                syn::Item::Static(_item_static) => todo!("Static"),
                syn::Item::Trait(_item_trait) => todo!("Trait"),
                syn::Item::TraitAlias(_item_trait_alias) => todo!("TraitAlias"),
                syn::Item::Type(_item_type) => todo!("Type"),
                syn::Item::Union(_item_union) => todo!("Union"),
                syn::Item::Use(_item_use) => panic!("Imports should have been stripped before this stage!"),
                syn::Item::Verbatim(_token_stream) => panic!("Cannot handle verbatim syntax items here!"),
                _ => todo!("Unsupported syntax item"),
            }
        }

        let globals = self.globals.into_iter().map(|a| a.1).collect::<Vec<_>>();
        let structs = self.structs.into_iter().map(|a| a.1.element).collect::<Vec<_>>();
        let functions = self.functions.into_iter().map(|a| a.1).collect::<Vec<_>>();

        let mut output = String::new();
        let replacements = HashMap::new();
        output.push_str(&ShaderElement::to_wgsl(&globals, &replacements, false));
        output.push_str("\n");
        output.push_str(&ShaderElement::to_wgsl(&structs, &replacements, false));
        output.push_str("\n");
        output.push_str(&ShaderElement::to_wgsl(&functions, &replacements, false));
        return output;
    }

    pub fn transpile_naga(self) -> Result<(naga::Module, naga::valid::ModuleInfo), Box<dyn std::error::Error>> {
        let wgsl_str = self.transpile_raw();
        let module = naga::front::wgsl::parse_str(&wgsl_str)
            .map_err(|e| {
                eprintln!("{}", e.emit_to_string(&wgsl_str));
                e
            })?;
        let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
        let info = validator.validate(&module)?;
        Ok((module, info))
    }

    pub fn transpile_wgsl(self) -> Result<String, Box<dyn std::error::Error>> {
        let (module, info) = self.transpile_naga()?;
        let mut out = String::new();
        let mut writer = naga::back::wgsl::Writer::new(&mut out, naga::back::wgsl::WriterFlags::empty());
        writer.write(&module, &info).expect("Failed to write WGSL");
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fragment output struct every test shader returns.
    const OUT: &str = "struct Out { #[location = 0] color: Vec4 }";

    /// Transpiles `src` (prefixed with `OUT`) with `main` as its fragment entry point,
    /// returning the raw and the naga validated WGSL.
    fn transpile(src: &str) -> (String, String) {
        let file = syn::parse_file(&format!("{OUT}\n{src}")).expect("test shader must parse");
        let raw = Transpiler::new(file.clone(), "main".into(), "fragment".into()).transpile_raw();
        let wgsl = Transpiler::new(file, "main".into(), "fragment".into())
            .transpile_wgsl()
            .unwrap_or_else(|err| panic!("invalid WGSL: {err}\n{raw}"));
        return (raw, wgsl);
    }

    #[test]
    fn literal_suffixes_become_wgsl_suffixes() {
        let (raw, _) = transpile("fn main() -> Out {
            let a = 1u32;
            let b = 2i32;
            let c = 1.5f32;
            let d = 3f32;
            let e = 0x10u32;
            let f: f32 = 2.0f64;
            return Out { color: Vec4::new(c, d, f, 1.0) };
        }");
        for expected in ["= 1u;", "= 2i;", "= 1.5f;", "= 3f;", "= 16u;", "= 2.0;"] {
            assert!(raw.contains(expected), "missing {expected:?} in\n{raw}");
        }
    }

    #[test]
    fn trailing_expressions_are_returned() {
        transpile("fn half(v: f32) -> f32 { v * 0.5 }
            fn main() -> Out { Out { color: Vec4::splat(half(1.0)) } }");
    }

    #[test]
    fn trailing_control_flow_is_not_returned() {
        transpile("fn pick(v: f32) -> f32 { if v > 0.5 { return 1.0; } else { return 0.0; } }
            fn main() -> Out { return Out { color: Vec4::splat(pick(0.7)) }; }");
    }

    #[test]
    fn matrix_identity_and_zero_constructors() {
        let (_, wgsl) = transpile("fn main() -> Out {
            let m = Mat4::identity();
            let z = Mat3::zero();
            let v = z * Vec3::splat(1.0);
            return Out { color: m * Vec4::from_vec3_w(v, 1.0) };
        }");
        assert!(wgsl.contains("mat4x4<f32>(vec4<f32>(1f, 0f, 0f, 0f)"), "{wgsl}");
        assert!(wgsl.contains("mat3x3<f32>()"), "{wgsl}");
    }

    #[test]
    fn constructors_keep_their_scalar_type() {
        transpile("fn main() -> Out {
            let v = UVec2::new(1, 2);
            let s: u32 = v.x();
            let i = IVec3::splat(-1);
            return Out { color: Vec4::splat(1.0) };
        }");
    }

    #[test]
    fn multi_word_builtins_become_camel_case() {
        let (raw, _) = transpile("fn main() -> Out {
            let v = Vec3::new(1.0, 2.0, 3.0);
            let a = inverse_sqrt(4.0);
            let b = face_forward_vec3(v, v, v);
            let c = mix_vec3_t(v, v, v);
            let d = pow_vec3_cw(v, v);
            let e = count_one_bits_u32(7u32);
            let f = extract_bits_u32(7u32, 0u32, 2u32);
            let g = first_leading_bit_i32(5i32);
            let h = quantize_to_f16(1.0);
            let i = round_wgsl(1.5);
            return Out { color: Vec4::from_vec3_w(b + c + d, a + h + i) };
        }");
        for expected in [
            "inverseSqrt(", "faceForward(", "mix(", "pow(", "countOneBits(",
            "extractBits(", "firstLeadingBit(", "quantizeToF16(", "round(",
        ] {
            assert!(raw.contains(expected), "missing {expected:?} in\n{raw}");
        }
    }

    #[test]
    fn comparison_functions_become_operators() {
        transpile("fn main() -> Out {
            let a = Vec3::splat(1.0);
            let b = Vec3::splat(2.0);
            let lt = less_than_vec3(a, b);
            let eq = equal_vec4(Vec4::splat(1.0), Vec4::splat(1.0));
            let ge = greater_than_equal_vec2(Vec2::splat(1.0), Vec2::splat(0.0));
            if all_bvec3(lt) && any_bvec4(eq) && all_bvec2(ge) {
                return Out { color: Vec4::splat(1.0) };
            }
            return Out { color: Vec4::splat(0.0) };
        }");
    }

    #[test]
    fn user_functions_keep_snake_case_names() {
        let (raw, _) = transpile("fn tint_vec3(v: Vec3) -> Vec3 { return v * 0.5; }
            fn main() -> Out { return Out { color: Vec4::from_vec3_w(tint_vec3(Vec3::splat(1.0)), 1.0) }; }");
        assert!(raw.contains("tint_vec3("), "{raw}");
    }

    #[test]
    fn match_or_patterns_and_expression_arms() {
        let (raw, _) = transpile("fn main() -> Out {
            let i = 2;
            let mut x = 0.0;
            match i {
                1 | 2 => x = 1.0,
                3 => { x = 0.5; }
                _ => { x = 0.0; }
            }
            return Out { color: Vec4::splat(x) };
        }");
        assert!(raw.contains("case 1, 2:"), "{raw}");
    }

    #[test]
    fn match_wildcard_alongside_values() {
        let (raw, _) = transpile("fn main() -> Out {
            let i = 2u32;
            let mut x = 0.0;
            match i { 1u32 => { x = 1.0; } 4u32 | _ => { x = 0.5; } }
            return Out { color: Vec4::splat(x) };
        }");
        assert!(raw.contains("case 4u, default:"), "{raw}");
    }

    #[test]
    fn casts_convert_types() {
        let (raw, _) = transpile("fn main() -> Out {
            let values = [1.0, 2.0, 3.0];
            let i = 1u32;
            let f = i as f32;
            let v = values[i as usize];
            return Out { color: Vec4::splat(f + v) };
        }");
        assert!(raw.contains("f32(i)"), "{raw}");
        assert!(raw.contains("values[i]"), "{raw}");
    }

    #[test]
    fn compute_shaders_with_storage_and_workgroup_memory() {
        let file = syn::parse_file("
            #[derive(ShaderLayout)]
            struct Ids { #[builtin = \"workgroup_id\"] wg: UVec3, #[builtin = \"local_invocation_index\"] lid: u32 }

            #[derive(ShaderGroup)]
            struct Bindings {
                #[uniform] dims: UVec4,
                #[read] w: RuntimeArray<u32>,
                #[write] y: RuntimeArray<f32>,
                #[workgroup] partial: [f32; 64],
            }

            fn main(#[group = 0] mut b: Bindings, ids: Ids) {
                let packed = b.w[ids.wg.x() as usize];
                b.partial[ids.lid as usize] = bitcast_f32(packed << 16u32);
                workgroup_barrier();
                if ids.lid == 0u32 { b.y[ids.wg.x() as usize] = b.partial[0] + bitcast_f32(b.dims.x()); }
            }
        ").expect("test shader must parse");
        let raw = Transpiler::new(file.clone(), "main".into(), "compute(64)".into()).transpile_raw();
        Transpiler::new(file, "main".into(), "compute(64)".into())
            .transpile_wgsl()
            .unwrap_or_else(|err| panic!("invalid WGSL: {err}\n{raw}"));
        for expected in [
            "@compute\n@workgroup_size(64)",
            "@group(0) @binding(1) var<storage, read> w: array<u32>",
            "@group(0) @binding(2) var<storage, read_write> y: array<f32>",
            "var<workgroup> partial: array<f32, 64>",
            "bitcast<f32>(",
            "workgroupBarrier()",
        ] {
            assert!(raw.contains(expected), "missing {expected:?} in\n{raw}");
        }
    }

    #[test]
    fn color_swizzles() {
        transpile("fn main() -> Out {
            let c = Vec4::new(1.0, 0.5, 0.25, 1.0);
            return Out { color: Vec4::from_vec3_w(c.rgb() * c.a(), c.r()) };
        }");
    }
}
