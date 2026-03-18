use std::error::Error;
use std::collections::{HashMap, HashSet};

use crate::*;

pub const PROCESSOR_ATTRIBUTES: &[&str] = &["main", "public"];

/// Storage object for all needed data for a pre-compiled shader.
#[derive(Default, Debug, Clone)]
pub struct ShaderFile {
    pub name: String,
    pub elements: Vec<ShaderElement>,
    pub imports: HashSet<String>
}

impl ShaderFile {
    /// Creates a new `ShaderFile` from the given name, `elements` and `imports`.
    pub fn new(
        name: impl Into<String>,
        elements: Vec<ShaderElement>,
        imports: HashSet<String>
    ) -> Self {
        Self { name: name.into(), elements, imports }
    }

    /// Parses the given `src` into a `ShaderFile` of the given `name`.
    pub fn parse(
        name: impl Into<String>,
        src: impl Into<String>
    ) -> Result<Self, Box<dyn Error>> {
        let result = ShaderElement::parse(src)?;
        Ok(Self { 
            name: result.name.unwrap_or_else(|| name.into()), 
            elements: result.elements, 
            imports: result.imports 
        })
    }
}

/// Container object for an attribute in a shader.
/// Example: "@binding(0)"
#[derive(Debug, Clone)]
pub struct Attr {
    pub name: String,
    pub content: String
}

/// Container object for a parameter in a shader.
/// Example: "test: array<u32, 32u>"
#[derive(Debug, Clone)]
pub struct Param {
    pub attrs: Vec<Attr>,
    pub name: String,
    pub ty: String
}

/// Container object for all possible root elements of a shader.
/// Struct - Wgsl struct
/// Function - Wgsl function
/// Global - Global variable
#[derive(Debug, Clone)]
pub enum ShaderElement {
    Struct {
        attrs: Vec<Attr>,
        name: String,
        params: Vec<Param>
    },
    Function {
        attrs: Vec<Attr>,
        name: String,
        params: Vec<Param>,
        ret_ty: Option<String>,
        block: String,
        preprocessor_instructions: Vec<String>
    },
    Global {
        attrs: Vec<Attr>,
        declared_as: String,
        name: String,
        ty: String
    },
    PreprocessorInstruction {
        raw: String,
    }
}

#[derive(Debug)]
pub enum ShaderPreProcessorError {
    ParseError(String),
    UnexpectedToken(String),
    InvalidSyntax(String),
    UnknownImport(String)
}

impl std::fmt::Display for ShaderPreProcessorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShaderPreProcessorError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ShaderPreProcessorError::UnexpectedToken(msg) => write!(f, "Unexpected token: {}", msg),
            ShaderPreProcessorError::InvalidSyntax(msg) => write!(f, "Invalid syntax: {}", msg),
            ShaderPreProcessorError::UnknownImport(msg) => write!(f, "Unknown import: {}", msg)
        }
    }
}

impl Error for ShaderPreProcessorError {}

impl ShaderElement {
    /// Parses a `src` string into its wgsl `elements`.
    pub fn parse(src: impl Into<String>) -> Result<ParserResult, Box<dyn Error>> {
        let src = src.into();
        let mut parser = Parser::new(&src);
        parser.parse_all_elements()
    }

    /// Converts a single `ShaderElement` with the given replacements for # marked defines
    /// into a single shader string.
    pub fn single_to_wgsl(&self, replacements: &HashMap<String, String>, only_public: bool) -> String {
        match self {
            ShaderElement::Struct { attrs, name, params } => {
                let mut output = String::new();

                if only_public && !attrs.iter().any(|attr| attr.name == "public") { return String::new(); }
                
                // Add attributes
                for attr in attrs {
                    if PROCESSOR_ATTRIBUTES.contains(&attr.name.as_str()) { continue }

                    if attr.content.is_empty() {
                        output.push_str(&format!("@{} ", attr.name));
                    } else {
                        output.push_str(&format!("@{}({}) ", attr.name, attr.content));
                    }
                }
                
                output.push_str(&format!("struct {} {{\n", name));
                
                // Add fields
                for param in params {
                    output.push_str("    ");
                    
                    // Add field attributes
                    for attr in &param.attrs {
                        if attr.content.is_empty() {
                            output.push_str(&format!("@{} ", attr.name));
                        } else {
                            output.push_str(&format!("@{}({}) ", attr.name, attr.content));
                        }
                    }
                    
                    output.push_str(&format!("{}: {},\n", param.name, param.ty));
                }
                
                output.push_str("};\n");
                output
            }
            
            ShaderElement::Function { attrs, name, params, block, ret_ty, preprocessor_instructions: _ } => {
                let mut output = String::new();

                if only_public && !attrs.iter().any(|attr| attr.name == "public") { return String::new(); }
                
                // Add attributes
                for attr in attrs {
                    if PROCESSOR_ATTRIBUTES.contains(&attr.name.as_str()) { continue }

                    if attr.content.is_empty() {
                        output.push_str(&format!("@{}\n", attr.name));
                    } else {
                        output.push_str(&format!("@{}({})\n", attr.name, attr.content));
                    }
                }
                
                output.push_str(&format!("fn {}(\n", name));
                
                // Add parameters
                for (i, param) in params.iter().enumerate() {
                    output.push_str("    ");
                    
                    // Add parameter attributes
                    for attr in &param.attrs {
                        if attr.content.is_empty() {
                            output.push_str(&format!("@{} ", attr.name));
                        } else {
                            output.push_str(&format!("@{}({}) ", attr.name, attr.content));
                        }
                    }
                    
                    output.push_str(&format!("{}: {}", param.name, param.ty));
                    
                    if i < params.len() - 1 {
                        output.push_str(",\n");
                    } else {
                        output.push('\n');
                    }
                }
                
                output.push_str(") ");

                if let Some(ret_ty) = ret_ty {
                    output.push_str(&format!("-> {ret_ty} "));
                }
                
                // Replace preprocessor instructions in block
                let mut replaced_block = block.clone();
                for (key, value) in replacements {
                    replaced_block = replaced_block.replace(key, value);
                }
                
                output.push_str(&replaced_block);
                output.push('\n');
                output
            }
            
            ShaderElement::Global { attrs, declared_as, name, ty } => {
                let mut output = String::new();

                if only_public && !attrs.iter().any(|attr| attr.name == "public") { return String::new(); }
                
                // Add attributes
                for attr in attrs {
                    if PROCESSOR_ATTRIBUTES.contains(&attr.name.as_str()) { continue }

                    if attr.content.is_empty() {
                        output.push_str(&format!("@{} ", attr.name));
                    } else {
                        output.push_str(&format!("@{}({}) ", attr.name, attr.content));
                    }
                }
                
                // Handle var<storage_class> syntax
                if declared_as == "var" && ty.starts_with('<') {
                    let split_pos = ty.find('>').unwrap_or(0) + 1;
                    let storage = &ty[..split_pos];
                    let actual_ty = &ty[split_pos..].trim_start();
                    output.push_str(&format!("var{} {}: {};\n", storage, name, actual_ty));
                } else {
                    output.push_str(&format!("{} {}: {};\n", declared_as, name, ty));
                }
                
                output
            }
            
            ShaderElement::PreprocessorInstruction { raw } => {
                // // Look up replacement in map
                // if let Some(replacement) = replacements.get(raw) {
                //     format!("{}\n", replacement)
                // } else {
                //     // If no replacement found, return original with newline
                //     format!("{}\n", raw)
                // }
                todo!("Instruction {raw:?}")
            }
        }
    }
    
    /// Converts a multiple `ShaderElement` with the given replacements for # marked defines
    /// into a single shader string.
    pub fn to_wgsl(elements: &[ShaderElement], replacements: &HashMap<String, String>, only_public: bool) -> String {
        let mut output = String::new();
        
        for (i, element) in elements.iter().enumerate() {
            output.push_str(&element.single_to_wgsl(replacements, only_public));
            
            // Add extra newline between elements for readability
            if i < elements.len() - 1 {
                output.push('\n');
            }
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_struct() {
        let src = r#"
        struct VertexInput {
            @location(0) position: vec3<f32>,
            @location(1) uvs: vec2<f32>,
        };
        "#;
        
        let result = ShaderElement::parse(src).unwrap();
        let result = result.elements.get(0).unwrap();
        match result {
            ShaderElement::Struct { name, params, .. } => {
                assert_eq!(name, "VertexInput");
                assert_eq!(params.len(), 2);
                assert_eq!(params[0].name, "position");
                assert_eq!(params[0].ty, "vec3<f32>");
            }
            _ => panic!("Expected struct")
        }

        let wgsl = result.single_to_wgsl(&HashMap::new(), false);
        println!("WGSL: {:?}", wgsl)
    }
    
    #[test]
    fn test_parse_global() {
        let src = "@group(0) @binding(0) var<uniform> bones: array<mat4x4<f32>, 32u>;";
        
        let result = ShaderElement::parse(src).unwrap();
        let result = result.elements.get(0).unwrap();
        
        match result {
            ShaderElement::Global { name, ty, attrs, .. } => {
                assert_eq!(name, "bones");
                assert_eq!(ty, "array<mat4x4<f32>,32u>");
                assert_eq!(attrs.len(), 2);
            }
            _ => panic!("Expected global")
        }
    }

    #[test]
    fn test_preprocessor_instructions() {
        let src = r#"
#import skeletal_data::VertexInput

struct MyStruct {
    @location(0) position: vec3<f32>,
};
        "#;
        
        let result = ShaderElement::parse(src).unwrap();
        
        // verify imports
        assert!(result.imports.contains("skeletal_data"));
        
        // verify single struct
        match &result.elements[0] {
            ShaderElement::Struct { name, .. } => {
                assert_eq!(name, "MyStruct");
            }
            _ => panic!("Expected struct")
        }
    }
    
    #[test]
    fn test_preprocessor_in_code() {
        let src = r#"
fn test() {
    if (#def == 3) {
        var test = #{def_value};
    }
}
        "#;
        
        let result = ShaderElement::parse(src).unwrap();
        
        match &result.elements[0] {
            ShaderElement::Function { block, preprocessor_instructions, .. } => {
                // The block should contain the preprocessor syntax as-is
                assert!(block.contains("#def"));
                assert!(block.contains("#{def_value}"));
                
                // Check extracted instructions
                assert_eq!(preprocessor_instructions.len(), 2);
                assert!(preprocessor_instructions.contains(&"#def".to_string()));
                assert!(preprocessor_instructions.contains(&"#{def_value}".to_string()));
            }
            _ => panic!("Expected function")
        }
    }
    
    #[test]
    fn test_preprocessor_replacement() {
        let src = r#"
fn test() {
    if (#def == 3) {
        var test = #{def_value};
    }
}
        "#;
        
        let mut replacements = HashMap::new();
        replacements.insert("#def".to_string(), "my_constant".to_string());
        replacements.insert("#{def_value}".to_string(), "42".to_string());
        
        let result = ShaderElement::parse(src).unwrap();
        let wgsl = ShaderElement::to_wgsl(&result.elements, &replacements, false);
        
        assert!(wgsl.contains("my_constant"));
        assert!(wgsl.contains("= 42;"));
        assert!(!wgsl.contains("#def"));
        assert!(!wgsl.contains("#{def_value}"));
    }
    
    #[test]
    fn test_preprocessor_ignores_comments() {
        let src = r#"
fn test() {
    // This #fake_def should be ignored
    var real = #real_def;
    /* 
       This #also_fake should be ignored
       And #{fake_expr} too
    */
    var another = #{real_expr};
}
        "#;
        
        let result = ShaderElement::parse(src).unwrap();
        
        match &result.elements[0] {
            ShaderElement::Function { preprocessor_instructions, .. } => {
                // Should only find the real preprocessor instructions
                assert_eq!(preprocessor_instructions.len(), 2);
                assert!(preprocessor_instructions.contains(&"#real_def".to_string()));
                assert!(preprocessor_instructions.contains(&"#{real_expr}".to_string()));
                
                // Should NOT contain the commented ones
                assert!(!preprocessor_instructions.contains(&"#fake_def".to_string()));
                assert!(!preprocessor_instructions.contains(&"#also_fake".to_string()));
                assert!(!preprocessor_instructions.contains(&"#{fake_expr}".to_string()));
            }
            _ => panic!("Expected function")
        }
    }

    // #[test]
    // pub fn test_parse_large() {
    //     let src = include_str!("../../skeletal/shaders/vertex.wgsl");
    //     let result = ShaderElement::parse(src).unwrap();
    //     let _result = ShaderElement::to_wgsl(&result.elements, &HashMap::new());
    // }
}
