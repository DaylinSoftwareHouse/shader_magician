use std::{collections::{HashMap, HashSet, LinkedList}, error::Error, hash::{Hash, Hasher}};

use ahash::AHasher;

use crate::{Attr, BlockParser, Param, ShaderElement, ShaderFile};

/// The default build instruction used in `CompilationInstructions`.
pub const DEFAULT_BUILD_INSTRUCTION: &'static BuildInstructions<'static> = &BuildInstructions {
    main_attribute: "@main",
    main_fn_name: "main",
    input_types: &["f32"],
    output_type: "f32"
};

/// Shader composer that serves as a central storage
/// to store shader files and compilated shaders.
#[derive(Default, Debug, Clone)]
pub struct ShaderComposer {
    files: HashMap<String, ShaderFile>,
    compile_cache: HashMap<u64, String>
}

/// Instructions meant for whole groups of shaders like vertex or
/// fragment shaders.  This tells the shader how to build the final
/// main functions for a shader type.
/// 
/// Parameters:
/// * main_attribute: The attribute to apply to the main function to
///     be generated like "@vertex" or "@fragment".
/// * main_fn_name: The reserved name for the main function like "fs_final_main".
/// * input_types: The list of types that the main function should
///     take and be given to the base shader and each modifiers main 
///     functions as arguments.
/// * output_type: The final output type of the generated main function.
#[derive(Default, Hash)]
pub struct BuildInstructions<'a> {
    pub main_attribute: &'a str,
    pub main_fn_name: &'a str,
    pub input_types: &'a [&'a str],
    pub output_type: &'a str
}

/// Instructions meant for each individual compilation.  This tells
/// the composer what shaders to compose into a single shader source
/// code.
/// 
/// Parameters:
/// * shader: The name of the shader that is the "base" shader of this shader.
/// * modifiers: The modifiers to apply to the output of the "base" shader.
/// * import_rewrites: The import names to rewrite like vertex to the actual vertex 
///     shaders name.
/// * defs: The key value pairs of all definitions that may be applied to this file.
/// * prefix: A list of `ShaderElement`s that will be applied at the start of this file.  
///     This is useful to apply things like framebuffer output structs to the start of 
///     the file without the shader needing to define them.
/// * instructions: The build instructions that apply to this instruction.
#[derive(Clone, Copy, Hash)]
pub struct CompilationInstructions<'a> {
    pub shaders: &'a [String],
    pub import_rewrites: &'a [(String, String)],
    pub defs: &'a [(String, String)],
    pub prefix: &'a [ShaderElement],
    pub instructions: &'a BuildInstructions<'a>
}

impl <'a> Default for CompilationInstructions<'a> {
    fn default() -> Self {
        Self {
            shaders: &[],
            import_rewrites: &[],
            defs: &[],
            prefix: &[],
            instructions: DEFAULT_BUILD_INSTRUCTION
        }
    }
}

struct ImportInstruction {
    filename: String,
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
        instructions: CompilationInstructions
    ) -> &String {
        // complete intos
        let shaders = instructions.shaders;
        let mut import_rewrites = instructions.import_rewrites.iter().cloned().collect::<HashMap<_, _>>();
        let defs = instructions.defs;
        let build_instuctions = instructions.instructions;

        // hash key
        let mut hasher = AHasher::default();
        instructions.hash(&mut hasher);
        let cache_key = hasher.finish();

        // pull from cache if exists
        self.compile_cache
            .entry(cache_key)
            .or_insert_with(|| {
                // setup output and replacements
                let mut output = String::new();
                let replacements = defs.iter().cloned().collect::<HashMap<_, _>>();

                // create initial import list ot compile
                let mut imported = HashSet::<String>::new();
                let mut to_import = LinkedList::<ImportInstruction>::new();

                // add mods to import list
                for shader in shaders {
                    imported.insert(shader.to_string());
                    to_import.push_back(ImportInstruction { filename: shader.to_string(), only_public: false });
                }

                let output_type = build_instuctions.output_type.to_string();
                let mut output_type_def_function = Option::<String>::None;
                let mut processors = Vec::<String>::new();

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
                    
                    // attempt to find default function for output_type
                    if let Some(local_def_function) = file.elements
                        .iter()
                        .filter_map(|element| match element {
                            ShaderElement::Function { attrs, name, params: _, ret_ty: _, block: _, preprocessor_instructions: _ } => {
                                if attrs.iter().any(|a| a.name == "public") && attrs.iter().any(|a| a.name == "default" && a.content == output_type) {
                                    Some(name.clone())
                                } else {
                                    None
                                }
                            },
                            _ => None
                        })
                        .next() 
                    {
                        output_type_def_function = Some(local_def_function);
                    }

                    if let Some(local_main) = local_main {
                        processors.push(local_main);
                    }

                    // save next imports
                    for import in &file.imports {
                        if imported.contains(import) { continue }
                        imported.insert(import.clone());
                        to_import.push_back(ImportInstruction { filename: import.clone(), only_public: true });
                    }

                    // convert to wgsl and save to output
                    output.insert_str(0, &ShaderElement::to_wgsl(&file.elements, &replacements, import.only_public));
                }

                // look for processor functions and defaults in prefix
                {
                    let local_main = instructions.prefix.iter()
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

                    let local_default = instructions.prefix.iter()
                        .filter_map(|element| match element {
                            ShaderElement::Function { attrs, name, params: _, ret_ty: _, block: _, preprocessor_instructions: _ } => {
                                if attrs.iter().any(|a| a.name == "public") && attrs.iter().any(|a| a.name == "default" && a.content == output_type) {
                                    Some(name.clone())
                                } else {
                                    None
                                }
                            },
                            _ => None
                        })
                        .next();

                    if let Some(local_main) = local_main {
                        processors.push(local_main);
                    }

                    if let Some(local_default) = local_default {
                        output_type_def_function = Some(local_default);
                    }
                }

                // compute parameter names
                let params = (0 .. build_instuctions.input_types.len())
                    .map(|idx| format!("v{}", idx))
                    .collect::<Vec<_>>()
                    .join(", ");

                // compute typed parameters
                let params_typed = (0 .. build_instuctions.input_types.len())
                    .map(|idx| Param {
                        attrs: vec![],
                        name: format!("v{}", idx),
                        ty: build_instuctions.input_types[idx].to_string()
                    })
                    .collect::<Vec<_>>();

                // compute main function code
                let mut mfc = String::new();
                mfc.push_str("{\n");
                mfc.push_str(&format!("    var result = {}();\n", output_type_def_function.expect(format!("No default function found for {}", output_type).as_str())));
                // mfc.push_str(&format!("    result = {}({}, result);\n", main_function.expect("No main function provided in main shader"), params));
                for mod_function in processors.drain(..) {
                    mfc.push_str(&format!("    result = {}({}, result);\n", mod_function, params))
                }
                mfc.push_str("    return result;\n}");
                let mfc_block = BlockParser::new(&mfc).parse_block();
                let Ok(mfc_block) = mfc_block else { panic!("Failed to load mfc function"); };

                // build main function
                let main_function = ShaderElement::Function { 
                    attrs: vec![Attr { name: build_instuctions.main_attribute.to_string(), content: String::new() }], 
                    name: build_instuctions.main_fn_name.to_string(), 
                    params: params_typed, 
                    ret_ty: Some(output_type), 
                    block: mfc_block, 
                    preprocessor_instructions: vec![] 
                };

                // add new main function to complete
                output.push_str(&main_function.single_to_wgsl(&replacements, false));
                return output;
            })
    }
}