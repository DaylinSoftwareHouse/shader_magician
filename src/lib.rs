pub mod elements;
pub mod parser;

use std::{collections::{HashMap, HashSet, LinkedList}, error::Error};

pub use elements::*;
pub(crate) use parser::*;

/// Shader composer that serves as a central storage
/// to store shader files and compilated shaders.
#[derive(Default, Debug, Clone)]
pub struct ShaderComposer {
    files: HashMap<String, ShaderFile>,
    compile_cache: HashMap<ShaderCacheKey, String>
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct ShaderCacheKey { names: Vec<String>, defs: Vec<(String, String)> }

impl ShaderComposer {
    /// Creates a new `ShaderComposer`.
    pub fn new() -> Self { Self::default() }

    /// Adds a new `ShaderFile` to this composer.
    pub fn add_file(&mut self, file: ShaderFile) -> Option<ShaderFile> {
        self.files.insert(file.name.clone(), file)
    }

    /// Stores a `ShaderFile` created from the given `name` and `src`.
    pub fn from_src(
        &mut self,
        name: impl Into<String>,
        src: impl Into<String>
    ) -> Result<Option<ShaderFile>, Box<dyn Error>> {
        let file = ShaderFile::parse(name, src)?;
        Ok(self.add_file(file))
    }

    /// Compiles the list of shader names and definitions into a single shader string.
    pub fn compile_to_str<'a>(
        &mut self,
        names: &[String],
        mut defs: Vec<(String, String)>
    ) -> &String {
        let cache_key = ShaderCacheKey { names: names.to_vec(), defs: defs.clone() };

        // pull from cache if exists
        self.compile_cache
            .entry(cache_key)
            .or_insert_with(|| {
                // setup output and replacements
                let mut output = String::new();
                let replacements = defs.drain(..).collect::<HashMap<_, _>>();

                // create initial import list ot compile
                let mut imported = HashSet::new();
                let mut to_import = LinkedList::new();
                for name in names {
                    if imported.contains(name) { continue }

                    let name = name.clone();
                    to_import.push_front(name.clone());
                    imported.insert(name);
                }

                // load all imports
                while let Some(import) = to_import.pop_front() {
                    // get file or throw error
                    let Some(file) = self.files.get(&import) else {
                        panic!("Unknown import {import:?}")
                    };

                    // save next imports
                    for import in &file.imports {
                        if imported.contains(import) { continue }
                        imported.insert(import.clone());
                        to_import.push_back(import.clone());
                    }

                    // convert to wgsl and save to output
                    output.push_str(&ShaderElement::to_wgsl(&file.elements, &replacements));
                }

                output
            })
    }
}
