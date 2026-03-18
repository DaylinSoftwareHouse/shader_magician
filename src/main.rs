use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use shader_magician::{BuildInstructions, ShaderComposer};


/**
 * This file contains the code for running the test structure.  We are doing this
 * outside of the usual unit tests architecture due to there large and external
 * nature.
 */


const VERTEX_BUILD_INSTRUCTIONS: BuildInstructions<'static> = BuildInstructions {
    main_attribute: "vertex",
    main_fn_name: "vs_final",
    input_types: &["VertexInput", "InstanceInput"],
    output_type: "VertexOutput"
};


const FRAGMENT_BUILD_INSTRUCTIONS: BuildInstructions<'static> = BuildInstructions {
    main_attribute: "fragment",
    main_fn_name: "fs_final",
    input_types: &["VertexOutput"],
    output_type: "vec4<f32>"
};


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
        lib_path.push("libs");
        let mut fs_mods_path = test_path.clone();
        fs_mods_path.push("fs_mods");
        let mut vs_mods_path = test_path.clone();
        vs_mods_path.push("vs_mods");
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
                let content = std::fs::read_to_string(lib.path())?;

                println!(" - Loading lib {file_stem:?}");

                composer.load_file_from_src(file_stem, content)?;
            }
        }

        // import fs_mods shaders
        let mut fs_mods = Vec::new();
        if fs_mods_path.exists() {
            for lib in std::fs::read_dir(&fs_mods_path)? {
                let Some(lib) = lib.ok() else { continue };
                let path = lib.path();
                if path.is_dir() { continue }
                let Some(file_stem) = path.file_stem().map(|a| a.to_str()).flatten() else { continue };
                let content = std::fs::read_to_string(lib.path())?;

                println!(" - Loading input shader {file_stem:?}");

                composer.load_file_from_src(file_stem, content)?;
                fs_mods.push(file_stem.to_string());
            }
        }

        // import fs_mods shaders
        let mut vs_mods = Vec::new();
        if vs_mods_path.exists() {
            for lib in std::fs::read_dir(&vs_mods_path)? {
                let Some(lib) = lib.ok() else { continue };
                let path = lib.path();
                if path.is_dir() { continue }
                let Some(file_stem) = path.file_stem().map(|a| a.to_str()).flatten() else { continue };
                let content = std::fs::read_to_string(lib.path())?;

                println!(" - Loading input shader {file_stem:?}");

                composer.load_file_from_src(file_stem, content)?;
                vs_mods.push(file_stem.to_string());
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

        // get vertex and fragment input paths
        let mut vs_in_path = test_path.clone();
        vs_in_path.push(metadata.import_rewrites.get("vertex").expect("Vertex shader not specified"));
        let mut fs_in_path = test_path.clone();
        fs_in_path.push(metadata.import_rewrites.get("fragment").expect("Fragment shader not specified"));

        // get vertex and fragment file stems
        let vs_file_stem = vs_in_path.file_stem().map(|a| a.to_str()).flatten().expect("Failed to get vertex file stem");
        let fs_file_stem = fs_in_path.file_stem().map(|a| a.to_str()).flatten().expect("Failed to get fragment file stem");

        println!(" - Compiling final shaders");

        // build vertex shader
        let vs_output = composer.compile(vs_file_stem, vs_mods, metadata.import_rewrites.clone(), vec![], VERTEX_BUILD_INSTRUCTIONS);
        let mut vs_output_path = out_path.clone();
        vs_output_path.push("vertex.wgsl");
        std::fs::write(vs_output_path, vs_output)?;

        // build fragment shader
        let fs_output = composer.compile(fs_file_stem, fs_mods, metadata.import_rewrites.clone(), vec![], FRAGMENT_BUILD_INSTRUCTIONS);
        let mut fs_output_path = out_path.clone();
        fs_output_path.push("fragment.wgsl");
        std::fs::write(fs_output_path, fs_output)?;
    }

    println!("Test shader generation complete!  Goodbye");

    Ok(())
}