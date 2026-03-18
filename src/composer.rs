use std::{collections::{HashMap, HashSet, LinkedList}, error::Error, hash::{Hash, Hasher}};

use ahash::AHasher;

use crate::{Attr, Param, ShaderElement, ShaderFile};

/// Shader composer that serves as a central storage
/// to store shader files and compilated shaders.
#[derive(Default, Debug, Clone)]
pub struct ShaderComposer {
    files: HashMap<String, ShaderFile>,
    compile_cache: HashMap<u64, String>
}

pub struct BuildInstructions<'a> {
    pub main_attribute: &'a str,
    pub main_fn_name: &'a str,
    pub input_types: &'a [&'a str],
    pub output_type: &'a str
}

struct ImportInstruction {
    filename: String,
    is_main: bool,
    is_mod: bool,
    only_public: bool
}

impl ShaderComposer {
    /// Creates a new `ShaderComposer`.
    pub fn new() -> Self { Self::default() }

    /// Returns true if a shader with the given name already exists in
    /// this composer.
    pub fn has_file(&self, name: impl Into<String>) -> bool {
        self.files.contains_key(&name.into())
    }

    /// Adds a new `ShaderFile` to this composer.
    pub fn add_file(&mut self, file: ShaderFile) -> Option<ShaderFile> {
        self.files.insert(file.name.clone(), file)
    }

    /// Stores a `ShaderFile` created from the given `name` and `src`.
    /// If a shader file with the same name already exists in this composer,
    /// it will be overriden.
    pub fn load_file_from_src(
        &mut self,
        name: impl Into<String>,
        src: impl Into<String>
    ) -> Result<Option<ShaderFile>, Box<dyn Error>> {
        let file = ShaderFile::parse(name, src)?;
        Ok(self.add_file(file))
    }

    /// Compiles a shader with the given definitions into a single shader string.
    pub fn compile<'a>(
        &mut self,
        shader: impl Into<String>,
        modifiers: impl Into<Vec<String>>,
        import_rewrites: impl Into<HashMap<String, String>>,
        defs: impl Into<Vec<(String, String)>>,
        instructions: BuildInstructions
    ) -> &String {
        // complete intos
        let shader = shader.into();
        let mut import_rewrites = import_rewrites.into();
        let mut defs = defs.into();
        let modifiers = modifiers.into();

        // hash key
        let mut hasher = AHasher::default();
        shader.hash(&mut hasher);
        defs.hash(&mut hasher);
        let cache_key = hasher.finish();

        // pull from cache if exists
        self.compile_cache
            .entry(cache_key)
            .or_insert_with(|| {
                // setup output and replacements
                let mut output = String::new();
                let replacements = defs.drain(..).collect::<HashMap<_, _>>();

                // create initial import list ot compile
                let mut imported = HashSet::<String>::new();
                let mut to_import = LinkedList::<ImportInstruction>::new();
                imported.insert(shader.clone());
                to_import.push_front(ImportInstruction { filename: shader.clone(), is_main: true, is_mod: false, only_public: false });

                // add mods to import list
                for modifier in modifiers {
                    imported.insert(modifier.clone());
                    to_import.push_back(ImportInstruction { filename: modifier, is_main: false, is_mod: true, only_public: false });
                }

                let mut main_function = Option::<String>::None;
                let mut mod_functions = Vec::<String>::new();

                // load all imports
                while let Some(mut import) = to_import.pop_front() {
                    if let Some(new_import) = import_rewrites.remove(&import.filename) {
                        import.filename = new_import;
                    }

                    // get file or throw error
                    let Some(file) = self.files.get(&import.filename) else {
                        panic!("Unknown import {:?}", import.filename)
                    };

                    // find main function of this file
                    let local_main = file.elements
                        .iter()
                        .filter_map(|element| match element {
                            ShaderElement::Function { attrs, name, params: _, ret_ty: _, block: _, preprocessor_instructions: _ } => {
                                if attrs.iter().any(|a| a.name == "main") {
                                    Some(name.clone())
                                } else {
                                    None
                                }
                            },
                            _ => None
                        })
                        .next();

                    // save to main or mod function lists
                    if import.is_main {
                        main_function = Some(local_main.expect("No main found in main shader file"));
                    } else if import.is_mod {
                        mod_functions.push(local_main.expect("No main found in mod shader file"));
                    }

                    // save next imports
                    for import in &file.imports {
                        if imported.contains(import) { continue }
                        imported.insert(import.clone());
                        to_import.push_back(ImportInstruction { filename: import.clone(), only_public: true, is_main: false, is_mod: false });
                    }

                    // convert to wgsl and save to output
                    output.insert_str(0, &ShaderElement::to_wgsl(&file.elements, &replacements, import.only_public));
                }

                // compute parameter names
                let params = (0 .. instructions.input_types.len())
                    .map(|idx| format!("v{}", idx))
                    .collect::<Vec<_>>()
                    .join(", ");

                // compute typed parameters
                let params_typed = (0 .. instructions.input_types.len())
                    .map(|idx| Param {
                        attrs: vec![],
                        name: format!("v{}", idx),
                        ty: instructions.input_types[idx].to_string()
                    })
                    .collect::<Vec<_>>();

                // compute main function code
                let mut mfc = String::new();
                mfc.push_str("{\n");
                mfc.push_str(&format!("    var result = {}({});\n", main_function.expect("No main function provided in main shader"), params));
                for mod_function in mod_functions.drain(..) {
                    mfc.push_str(&format!("    result = {}({}, result);\n", mod_function, params))
                }
                mfc.push_str("    return result;\n}");

                // build main function
                let main_function = ShaderElement::Function { 
                    attrs: vec![Attr { name: instructions.main_attribute.to_string(), content: String::new() }], 
                    name: instructions.main_fn_name.to_string(), 
                    params: params_typed, 
                    ret_ty: Some(instructions.output_type.to_string()), 
                    block: mfc, 
                    preprocessor_instructions: vec![] 
                };

                // add new main function to complete
                output.push_str(&main_function.single_to_wgsl(&replacements, false));
                return output;
            })
    }
}