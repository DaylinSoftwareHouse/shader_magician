use std::{collections::{HashMap, HashSet, LinkedList}, error::Error, hash::{Hash, Hasher}};

use ahash::AHasher;

use crate::{ShaderElement, ShaderFile};

/// Shader composer that serves as a central storage
/// to store shader files and compilated shaders.
#[derive(Default, Debug, Clone)]
pub struct ShaderComposer {
    files: HashMap<String, ShaderFile>,
    compile_cache: HashMap<u64, String>
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
        shader: impl Into<String>,
        import_rewrites: impl Into<HashMap<String, String>>,
        defs: impl Into<Vec<(String, String)>>
    ) -> &String {
        let shader = shader.into();

        let mut import_rewrites = import_rewrites.into();
        let mut defs = defs.into();

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
                to_import.push_front(ImportInstruction { filename: shader.clone(), only_public: false });

                // load all imports
                while let Some(mut import) = to_import.pop_front() {
                    if let Some(new_import) = import_rewrites.remove(&import.filename) {
                        import.filename = new_import;
                    }

                    // get file or throw error
                    let Some(file) = self.files.get(&import.filename) else {
                        panic!("Unknown import {:?}", import.filename)
                    };

                    // save next imports
                    for import in &file.imports {
                        if imported.contains(import) { continue }
                        imported.insert(import.clone());
                        to_import.push_back(ImportInstruction { filename: import.clone(), only_public: true });
                    }

                    // convert to wgsl and save to output
                    output.insert_str(0, &ShaderElement::to_wgsl(&file.elements, &replacements, import.only_public));
                }

                output
            })
    }
}