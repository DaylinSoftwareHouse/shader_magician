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
