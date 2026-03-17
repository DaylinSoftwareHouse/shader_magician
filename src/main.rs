use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use shader_magician::ShaderComposer;


/**
 * This file contains the code for running the test structure.  We are doing this
 * outside of the usual unit tests architecture due to there large and external
 * nature.
 */


#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub import_rewrites: HashMap<String, String>
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Created test shaders...");

    for test in std::fs::read_dir("./tests")? {
        let Some(test) = test.ok() else { continue };
        let test_path = test.path();
        let Some(test_name) = test_path.file_stem().map(|a| a.to_str()).flatten() else { continue };
        if !test.path().is_dir() { continue }
        println!(" - Testing {test_name:?}");

        let mut composer = ShaderComposer::new();

        // load paths
        let mut in_path = test_path.clone();
        in_path.push("in");
        let mut out_path = test_path.clone();
        out_path.push("out");
        let mut lib_path = test_path.clone();
        lib_path.push("lib");
        let mut meta_path = test_path.clone();
        meta_path.push("meta.toml");

        // import metadata
        let meta_content = std::fs::read_to_string(meta_path)?;
        let metadata = toml::from_str::<Metadata>(&meta_content)?;

        // import all libraries
        if lib_path.exists() {
            for lib in std::fs::read_dir(&lib_path)? {
                let Some(lib) = lib.ok() else { continue };
                let path = lib.path();
                if path.is_dir() { continue }
                let Some(file_stem) = path.file_stem().map(|a| a.to_str()).flatten() else { continue };
                let content = std::fs::read_to_string(file_stem)?;

                println!(" - Loading lib {file_stem:?}");

                composer.load_file_from_src(file_stem, content)?;
            }
        }

        // import all shaders
        if in_path.exists() {
            for lib in std::fs::read_dir(&in_path)? {
                let Some(lib) = lib.ok() else { continue };
                let path = lib.path();
                if path.is_dir() { continue }
                let Some(file_stem) = path.file_stem().map(|a| a.to_str()).flatten() else { continue };
                let content = std::fs::read_to_string(lib.path())?;

                println!(" - Loading input shader {file_stem:?}");

                composer.load_file_from_src(file_stem, content)?;
            }
        }

        // compile each shader
        if !out_path.exists() { std::fs::create_dir_all(&out_path)?; }
        for input in std::fs::read_dir(&in_path)? {
            let Some(input) = input.ok() else { continue };
            let path = input.path();
            if path.is_dir() { continue }
            let Some(file_stem) = path.file_stem().map(|a| a.to_str()).flatten() else { continue };

            let mut output_path = out_path.clone();
            output_path.push(format!("{file_stem}.wgsl"));

            println!(" - Compiling {file_stem:?}");
            let output = composer.compile(file_stem, metadata.import_rewrites.clone(), vec![]);
            std::fs::write(output_path, output)?;
        }
    }

    println!("Test shader generation complete!  Goodbye");

    Ok(())
}