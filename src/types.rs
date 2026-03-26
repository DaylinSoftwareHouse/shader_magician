use std::error::Error;

/// Represents a WGSL type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WgslType {
    /// Primitive types: f32, i32, u32, bool, f16
    Primitive(String),
    /// Vector types: vec2<T>, vec3<T>, vec4<T>
    Vector { size: u8, element: Box<WgslType> },
    /// Matrix types: mat2x2<T>, mat2x3<T>, etc.
    Matrix { cols: u8, rows: u8, element: Box<WgslType> },
    /// Array types: array<T, N> or array<T>
    Array { element: Box<WgslType>, size: Option<Box<WgslType>> },
    /// Binding array: binding_array<T>
    BindingArray { element: Box<WgslType> },
    /// Generic types with parameters: texture_2d<T>, sampler, etc.
    Generic { name: String, params: Vec<WgslType> },
    /// User-defined struct names
    Named(String),
    /// Pointer types: ptr<SC, T, AM>
    Pointer { storage_class: String, pointee: Box<WgslType>, access: Option<String> },
}

impl WgslType {
    /// Parse a WGSL type from a string.
    pub fn parse(input: &str) -> Result<Self, Box<dyn Error>> {
        let input = input.trim();
        let mut parser = TypeParser::new(input);
        parser.parse_type()
    }

    /// Convert this type back to WGSL string representation.
    pub fn to_wgsl(&self) -> String {
        match self {
            WgslType::Primitive(name) => name.clone(),
            WgslType::Vector { size, element } => {
                format!("vec{}<{}>", size, element.to_wgsl())
            }
            WgslType::Matrix { cols, rows, element } => {
                format!("mat{}x{}<{}>", cols, rows, element.to_wgsl())
            }
            WgslType::Array { element, size } => {
                match size {
                    Some(size) => format!("array<{}, {}>", element.to_wgsl(), size.to_wgsl()),
                    None => format!("array<{}>", element.to_wgsl()),
                }
            }
            WgslType::BindingArray { element } => {
                format!("binding_array<{}>", element.to_wgsl())
            }
            WgslType::Generic { name, params } => {
                if params.is_empty() {
                    name.clone()
                } else {
                    let params_str: Vec<String> = params.iter().map(|p| p.to_wgsl()).collect();
                    format!("{}<{}>", name, params_str.join(", "))
                }
            }
            WgslType::Named(name) => name.clone(),
            WgslType::Pointer { storage_class, pointee, access } => {
                match access {
                    Some(acc) => format!("ptr<{}, {}, {}>", storage_class, pointee.to_wgsl(), acc),
                    None => format!("ptr<{}, {}>", storage_class, pointee.to_wgsl()),
                }
            }
        }
    }
}

/// Helper struct for parsing types
struct TypeParser {
    input: String,
    pos: usize,
}

impl TypeParser {
    fn new(input: &str) -> Self {
        TypeParser {
            input: input.to_string(),
            pos: 0,
        }
    }

    fn parse_type(&mut self) -> Result<WgslType, Box<dyn Error>> {
        self.skip_whitespace();

        // Check for pointer type
        if self.peek_word() == "ptr" {
            return self.parse_pointer_type();
        }

        // Check for vec types
        if self.peek_word().starts_with("vec") && self.peek_word().len() == 4 {
            let size = self.peek_word().chars().nth(3).unwrap().to_digit(10).unwrap() as u8;
            if size >= 2 && size <= 4 {
                return self.parse_vector_type(size);
            }
        }

        // Check for mat types
        let word = self.peek_word();
        if word.starts_with("mat") && word.len() >= 5 {
            // Parse matNxM format
            if let Some(size) = self.parse_matrix_size(&word) {
                return self.parse_matrix_type(size.0, size.1);
            }
        }

        // Check for array
        if self.peek_word() == "array" {
            return self.parse_array_type();
        }

        // Check for binding_array
        if self.peek_word() == "binding_array" {
            return self.parse_binding_array_type();
        }

        // Check for primitive types
        let primitive_types = ["f32", "i32", "u32", "bool", "f16"];
        for prim in primitive_types {
            if self.peek_word() == prim {
                self.consume_word(prim)?;
                return Ok(WgslType::Primitive(prim.to_string()));
            }
        }

        // Otherwise, it's a named type or generic type
        self.parse_named_or_generic()
    }

    fn parse_matrix_size(&self, word: &str) -> Option<(u8, u8)> {
        // mat2x2, mat2x3, mat2x4, mat3x2, mat3x3, mat3x4, mat4x2, mat4x3, mat4x4
        if word.starts_with("mat") && word.len() == 5 {
            // matNx format (square matrix)
            let n = word.chars().nth(3)?.to_digit(10)? as u8;
            if n >= 2 && n <= 4 {
                return Some((n, n));
            }
        } else if word.starts_with("mat") && word.len() == 6 && word.chars().nth(4)? == 'x' {
            // matNxM format
            let n = word.chars().nth(3)?.to_digit(10)? as u8;
            let m = word.chars().nth(5)?.to_digit(10)? as u8;
            if n >= 2 && n <= 4 && m >= 2 && m <= 4 {
                return Some((n, m));
            }
        }
        None
    }

    fn parse_pointer_type(&mut self) -> Result<WgslType, Box<dyn Error>> {
        self.consume_word("ptr")?;
        self.skip_whitespace();
        self.expect_char('<')?;
        self.skip_whitespace();

        // Parse storage class
        let storage_class = self.consume_until_comma_or_gt()?;
        self.skip_whitespace();

        // Parse pointee type
        self.expect_char(',')?;
        self.skip_whitespace();
        let pointee = self.parse_type()?;

        // Optionally parse access mode
        self.skip_whitespace();
        let access = if self.peek_char() == Some(',') {
            self.consume_char(); // consume ','
            self.skip_whitespace();
            Some(self.consume_until_comma_or_gt()?)
        } else {
            None
        };

        self.expect_char('>')?;

        Ok(WgslType::Pointer {
            storage_class,
            pointee: Box::new(pointee),
            access,
        })
    }

    fn parse_vector_type(&mut self, size: u8) -> Result<WgslType, Box<dyn Error>> {
        let word = format!("vec{}", size);
        self.consume_word(&word)?;
        self.skip_whitespace();
        self.expect_char('<')?;
        self.skip_whitespace();

        let element = self.parse_type()?;

        self.skip_whitespace();
        self.expect_char('>')?;

        Ok(WgslType::Vector {
            size,
            element: Box::new(element),
        })
    }

    fn parse_matrix_type(&mut self, cols: u8, rows: u8) -> Result<WgslType, Box<dyn Error>> {
        // Consume matNxM or matN
        let word = if cols == rows {
            format!("mat{}", cols)
        } else {
            format!("mat{}x{}", cols, rows)
        };
        self.consume_word(&word)?;
        self.skip_whitespace();
        self.expect_char('<')?;
        self.skip_whitespace();

        let element = self.parse_type()?;

        self.skip_whitespace();
        self.expect_char('>')?;

        Ok(WgslType::Matrix {
            cols,
            rows,
            element: Box::new(element),
        })
    }

    fn parse_array_type(&mut self) -> Result<WgslType, Box<dyn Error>> {
        self.consume_word("array")?;
        self.skip_whitespace();
        self.expect_char('<')?;
        self.skip_whitespace();

        let element = self.parse_type()?;

        self.skip_whitespace();

        // Check for size
        let size = if self.peek_char() == Some(',') {
            self.consume_char(); // consume ','
            self.skip_whitespace();
            let size = self.parse_type()?;
            self.skip_whitespace();
            self.expect_char('>')?;
            Some(Box::new(size))
        } else {
            self.expect_char('>')?;
            None
        };

        Ok(WgslType::Array {
            element: Box::new(element),
            size,
        })
    }

    fn parse_binding_array_type(&mut self) -> Result<WgslType, Box<dyn Error>> {
        self.consume_word("binding_array")?;
        self.skip_whitespace();
        self.expect_char('<')?;
        self.skip_whitespace();

        let element = self.parse_type()?;

        self.skip_whitespace();
        self.expect_char('>')?;

        Ok(WgslType::BindingArray {
            element: Box::new(element),
        })
    }

    fn parse_named_or_generic(&mut self) -> Result<WgslType, Box<dyn Error>> {
        let name = self.consume_identifier()?;
        self.skip_whitespace();

        // Check for generic parameters
        if self.peek_char() == Some('<') {
            self.consume_char(); // consume '<'
            self.skip_whitespace();

            let mut params = Vec::new();

            loop {
                let param = self.parse_type()?;
                params.push(param);

                self.skip_whitespace();

                match self.peek_char() {
                    Some(',') => {
                        self.consume_char();
                        self.skip_whitespace();
                    }
                    Some('>') => {
                        self.consume_char();
                        break;
                    }
                    _ => return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Expected ',' or '>' in type parameters"
                    ))),
                }
            }

            Ok(WgslType::Generic { name, params })
        } else {
            Ok(WgslType::Named(name))
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input.chars().nth(self.pos)
    }

    fn peek_word(&self) -> String {
        let mut result = String::new();
        let mut pos = self.pos;

        while pos < self.input.len() {
            let c = self.input.chars().nth(pos).unwrap();
            if c.is_alphanumeric() || c == '_' {
                result.push(c);
                pos += 1;
            } else {
                break;
            }
        }

        result
    }

    fn consume_char(&mut self) -> Option<char> {
        let c = self.peek_char();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn consume_identifier(&mut self) -> Result<String, Box<dyn Error>> {
        let mut ident = String::new();

        while let Some(c) = self.peek_char() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(self.consume_char().unwrap());
            } else {
                break;
            }
        }

        if ident.is_empty() {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Expected identifier"
            )))
        } else {
            Ok(ident)
        }
    }

    fn consume_word(&mut self, expected: &str) -> Result<(), Box<dyn Error>> {
        let word = self.consume_identifier()?;
        if word == expected {
            Ok(())
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Expected '{}', got '{}'", expected, word)
            )))
        }
    }

    fn consume_until_comma_or_gt(&mut self) -> Result<String, Box<dyn Error>> {
        let mut result = String::new();

        while let Some(c) = self.peek_char() {
            if c == ',' || c == '>' {
                break;
            }
            result.push(self.consume_char().unwrap());
        }

        Ok(result.trim().to_string())
    }

    fn expect_char(&mut self, expected: char) -> Result<(), Box<dyn Error>> {
        match self.consume_char() {
            Some(c) if c == expected => Ok(()),
            Some(c) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Expected '{}', got '{}'", expected, c)
            ))),
            None => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Expected '{}', got end of input", expected)
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_type() {
        let ty = WgslType::parse("f32").unwrap();
        assert_eq!(ty.to_wgsl(), "f32");

        let ty = WgslType::parse("i32").unwrap();
        assert_eq!(ty.to_wgsl(), "i32");

        let ty = WgslType::parse("u32").unwrap();
        assert_eq!(ty.to_wgsl(), "u32");

        let ty = WgslType::parse("bool").unwrap();
        assert_eq!(ty.to_wgsl(), "bool");

        let ty = WgslType::parse("f16").unwrap();
        assert_eq!(ty.to_wgsl(), "f16");
    }

    #[test]
    fn test_vector_type() {
        let ty = WgslType::parse("vec3<f32>").unwrap();
        assert_eq!(ty.to_wgsl(), "vec3<f32>");

        let ty = WgslType::parse("vec4<i32>").unwrap();
        assert_eq!(ty.to_wgsl(), "vec4<i32>");

        let ty = WgslType::parse("vec2<u32>").unwrap();
        assert_eq!(ty.to_wgsl(), "vec2<u32>");
    }

    #[test]
    fn test_matrix_type() {
        let ty = WgslType::parse("mat4x4<f32>").unwrap();
        assert_eq!(ty.to_wgsl(), "mat4x4<f32>");

        let ty = WgslType::parse("mat2x3<f32>").unwrap();
        assert_eq!(ty.to_wgsl(), "mat2x3<f32>");

        let ty = WgslType::parse("mat3<f32>").unwrap();
        assert_eq!(ty.to_wgsl(), "mat3<f32>");
    }

    #[test]
    fn test_array_type() {
        let ty = WgslType::parse("array<f32>").unwrap();
        assert_eq!(ty.to_wgsl(), "array<f32>");

        let ty = WgslType::parse("array<f32, 10>").unwrap();
        assert_eq!(ty.to_wgsl(), "array<f32, 10>");

        let ty = WgslType::parse("array<vec3<f32>, 32u>").unwrap();
        assert_eq!(ty.to_wgsl(), "array<vec3<f32>, 32u>");

        let ty = WgslType::parse("array<mat4x4<f32>, 32u>").unwrap();
        assert_eq!(ty.to_wgsl(), "array<mat4x4<f32>, 32u>");
    }

    #[test]
    fn test_pointer_type() {
        let ty = WgslType::parse("ptr<storage, Data>").unwrap();
        assert_eq!(ty.to_wgsl(), "ptr<storage, Data>");

        let ty = WgslType::parse("ptr<storage, Data, read>").unwrap();
        assert_eq!(ty.to_wgsl(), "ptr<storage, Data, read>");

        let ty = WgslType::parse("ptr<uniform, vec3<f32>>").unwrap();
        assert_eq!(ty.to_wgsl(), "ptr<uniform, vec3<f32>>");
    }

    #[test]
    fn test_binding_array_type() {
        let ty = WgslType::parse("binding_array<f32>").unwrap();
        assert_eq!(ty.to_wgsl(), "binding_array<f32>");

        let ty = WgslType::parse("binding_array<texture_2d<f32>>").unwrap();
        assert_eq!(ty.to_wgsl(), "binding_array<texture_2d<f32>>");
    }

    #[test]
    fn test_named_type() {
        let ty = WgslType::parse("MyStruct").unwrap();
        assert_eq!(ty.to_wgsl(), "MyStruct");

        let ty = WgslType::parse("VertexInput").unwrap();
        assert_eq!(ty.to_wgsl(), "VertexInput");
    }

    #[test]
    fn test_generic_type() {
        let ty = WgslType::parse("texture_2d<f32>").unwrap();
        assert_eq!(ty.to_wgsl(), "texture_2d<f32>");

        let ty = WgslType::parse("texture_storage_2d<rgba32float, write>").unwrap();
        assert_eq!(ty.to_wgsl(), "texture_storage_2d<rgba32float, write>");
    }
}