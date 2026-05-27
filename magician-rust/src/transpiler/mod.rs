use std::{collections::HashMap, sync::atomic::AtomicU32};

use ahash::AHashMap;
use global::*;
use magician_ast::*;
use naga::valid::{Capabilities, ValidationFlags, Validator};

pub(crate) mod expr;
pub(crate) mod global;
pub(crate) mod stmts;

pub struct Transpiler {
    file: syn::File,
    globals: AHashMap<String, ShaderElement>,
    structs: AHashMap<String, TranspiledStruct>,
    functions: AHashMap<String, ShaderElement>,
    global_struct_names: Vec<String>
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
    pub fn new(file: syn::File) -> Self {
        Self {
            file,
            globals: AHashMap::new(),
            structs: AHashMap::new(),
            functions: AHashMap::new(),
            global_struct_names: Vec::new()
        }
    }

    pub fn transpile_raw(mut self) -> String {
        let counter = AtomicU32::new(0);

        for item in &self.file.items {
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
                        let globals = convert_global_struct(item_struct, &counter);
                        globals.into_iter().for_each(|(global_name, global)| {
                            self.globals.insert(global_name, global);
                        });
                        self.global_struct_names.push(item_struct.ident.to_string());
                    } else { 
                        let (struct_name, s) = convert_non_global_struct(item_struct);
                        self.structs.insert(struct_name, s);
                    }
                },
                
                syn::Item::Const(_item_const) => todo!(),
                syn::Item::Enum(_item_enum) => todo!(),
                syn::Item::ExternCrate(_item_extern_crate) => todo!(),
                syn::Item::Fn(item_fn) => { 
                    let function = convert_function(&self, item_fn, &self.global_struct_names);
                    self.functions.insert(function.0, function.1);
                },
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
