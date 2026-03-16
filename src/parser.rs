use std::{collections::HashSet, error::Error};

use crate::*;

pub(crate) struct Parser {
    input: String,
    pos: usize,
}

/// Contains the result of parsing via `ShaderElement::parse`.
/// Contains all the components of a shader pre-compilation.
/// This includes the shaders name, imports, and elements.
/// The name may not be filled.
#[derive(Default, Debug, Clone)]
pub struct ParserResult {
    pub elements: Vec<ShaderElement>,
    pub imports: HashSet<String>,
    pub name: Option<String>
}

impl Parser {
    pub(crate) fn new(input: &str) -> Self {
        Parser {
            input: input.to_string(),
            pos: 0,
        }
    }
    
    pub(crate) fn parse_all_elements(&mut self) -> Result<ParserResult, Box<dyn Error>> {
        let mut result = ParserResult::default();
        
        while !self.is_at_end() {
            self.skip_whitespace_and_comments();
            if self.is_at_end() { break; }

            let element = self.parse_element()?;

            match element {
                ShaderElement::PreprocessorInstruction { raw } => {
                    let tokens = raw.split(" ").collect::<Vec<_>>();

                    match tokens[0] {
                        "#import" => {
                            for idx in 1 .. tokens.len() {
                                let token = tokens[idx].split("::").next().unwrap();
                                result.imports.insert(token.to_string());
                            }
                        }
                        "#define_import_path" => {
                            let token = tokens[1];
                            result.name = Some(token.to_string());
                        }
                        _ => result.elements.push(ShaderElement::PreprocessorInstruction { raw }),
                    }
                },
                _ => result.elements.push(element),
            }
        }
        
        Ok(result)
    }
    
    pub(crate) fn parse_element(&mut self) -> Result<ShaderElement, Box<dyn Error>> {
        self.skip_whitespace_and_comments();
        
        // Check for preprocessor instruction
        if self.peek_char() == Some('#') {
            return self.parse_preprocessor_instruction();
        }
        
        // Parse attributes
        let attrs = self.parse_attributes()?;
        
        self.skip_whitespace_and_comments();
        
        // Check what kind of element this is
        if self.peek_word() == "struct" {
            self.parse_struct(attrs)
        } else if self.peek_word() == "fn" {
            self.parse_function(attrs)
        } else if self.peek_word() == "var" || self.peek_word() == "const" || self.peek_word() == "override" {
            self.parse_global(attrs)
        } else {
            Err(Box::new(ShaderPreProcessorError::ParseError(
                format!("Expected struct, fn, var, const, override, or preprocessor instruction, got: {}", self.peek_word())
            )))
        }
    }
    
    fn parse_preprocessor_instruction(&mut self) -> Result<ShaderElement, Box<dyn Error>> {
        // Consume the entire line starting with #
        let mut raw = String::new();
        
        while let Some(c) = self.peek_char() {
            if c == '\n' {
                self.consume_char(); // consume the newline
                break;
            }
            raw.push(self.consume_char().unwrap());
        }
        
        Ok(ShaderElement::PreprocessorInstruction { raw })
    }
    
    fn parse_attributes(&mut self) -> Result<Vec<Attr>, Box<dyn Error>> {
        let mut attrs = Vec::new();
        
        loop {
            self.skip_whitespace_and_comments();
            if !self.peek_char().map_or(false, |c| c == '@') {
                break;
            }
            
            self.consume_char(); // consume '@'
            let name = self.consume_identifier()?;
            
            self.skip_whitespace_and_comments();
            
            let content = if self.peek_char() == Some('(') {
                self.consume_char(); // consume '('
                let content = self.consume_until(')')?;
                self.consume_char(); // consume ')'
                content
            } else {
                String::new()
            };
            
            attrs.push(Attr { name, content });
        }
        
        Ok(attrs)
    }
    
    fn parse_struct(&mut self, attrs: Vec<Attr>) -> Result<ShaderElement, Box<dyn Error>> {
        self.consume_word("struct")?;
        self.skip_whitespace_and_comments();
        
        let name = self.consume_identifier()?;
        
        self.skip_whitespace_and_comments();
        self.expect_char('{')?;
        
        let mut params = Vec::new();
        
        loop {
            self.skip_whitespace_and_comments();
            
            if self.peek_char() == Some('}') {
                self.consume_char();
                break;
            }
            
            // Parse field attributes
            let field_attrs = self.parse_attributes()?;
            self.skip_whitespace_and_comments();
            
            let param_name = self.consume_identifier()?;
            self.skip_whitespace_and_comments();
            self.expect_char(':')?;
            self.skip_whitespace_and_comments();
            
            let param_ty = self.consume_type()?;
            
            params.push(Param {
                attrs: field_attrs,
                name: param_name,
                ty: param_ty
            });
            
            self.skip_whitespace_and_comments();
            
            if self.peek_char() == Some(',') {
                self.consume_char();
            }
        }
        
        self.skip_whitespace_and_comments();
        self.expect_char(';')?;
        
        Ok(ShaderElement::Struct {
            attrs,
            name,
            params
        })
    }
    
    fn parse_function(&mut self, attrs: Vec<Attr>) -> Result<ShaderElement, Box<dyn Error>> {
        self.consume_word("fn")?;
        self.skip_whitespace_and_comments();
        
        let name = self.consume_identifier()?;
        
        self.skip_whitespace_and_comments();
        self.expect_char('(')?;
        
        let mut params = Vec::new();
        
        loop {
            self.skip_whitespace_and_comments();
            
            if self.peek_char() == Some(')') {
                self.consume_char();
                break;
            }
            
            let param_name = self.consume_identifier()?;
            self.skip_whitespace_and_comments();
            self.expect_char(':')?;
            self.skip_whitespace_and_comments();
            
            let param_ty = self.consume_type()?;
            
            params.push(Param {
                attrs: Vec::new(),
                name: param_name,
                ty: param_ty
            });
            
            self.skip_whitespace_and_comments();
            
            if self.peek_char() == Some(',') {
                self.consume_char();
            }
        }
        
        self.skip_whitespace_and_comments();
        
        // Skip return type if present
        let mut ret_ty = None;
        if self.peek_char() == Some('-') {
            self.consume_until(' ')?;
            ret_ty = Some(self.consume_until('{')?);
        }
        
        self.skip_whitespace_and_comments();
        
        let block = self.consume_block()?;
        
        // Extract preprocessor instructions from the block
        let preprocessor_instructions = Self::extract_preprocessor_instructions(&block);
        
        Ok(ShaderElement::Function {
            attrs,
            name,
            params,
            block,
            ret_ty,
            preprocessor_instructions
        })
    }
    
    fn extract_preprocessor_instructions(block: &str) -> Vec<String> {
        let mut instructions = Vec::new();
        let mut chars = block.chars().peekable();
        let mut current_instruction = String::new();
        
        while let Some(c) = chars.next() {
            // Skip line comments
            if c == '/' && chars.peek() == Some(&'/') {
                chars.next(); // consume second '/'
                // Skip until end of line
                while let Some(ch) = chars.next() {
                    if ch == '\n' {
                        break;
                    }
                }
                continue;
            }
            
            // Skip block comments
            if c == '/' && chars.peek() == Some(&'*') {
                chars.next(); // consume '*'
                // Skip until we find */
                while let Some(ch) = chars.next() {
                    if ch == '*' && chars.peek() == Some(&'/') {
                        chars.next(); // consume '/'
                        break;
                    }
                }
                continue;
            }
            
            if c == '#' {
                // Check if this is #{...} syntax
                if chars.peek() == Some(&'{') {
                    chars.next(); // consume '{'
                    current_instruction.push_str("#{");
                    
                    // Collect until closing }
                    while let Some(ch) = chars.next() {
                        current_instruction.push(ch);
                        if ch == '}' {
                            instructions.push(current_instruction.clone());
                            current_instruction.clear();
                            break;
                        }
                    }
                } else {
                    // Regular #identifier syntax
                    current_instruction.push('#');
                    
                    // Collect identifier
                    while let Some(&ch) = chars.peek() {
                        if ch.is_alphanumeric() || ch == '_' {
                            current_instruction.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    
                    if current_instruction.len() > 1 {
                        instructions.push(current_instruction.clone());
                    }
                    current_instruction.clear();
                }
            }
        }
        
        instructions
    }
    
    fn parse_global(&mut self, attrs: Vec<Attr>) -> Result<ShaderElement, Box<dyn Error>> {
        let declared_as = self.consume_identifier().expect("Failed to parse global A");

        self.skip_whitespace_and_comments();
        
        // Handle var<storage_class>
        if self.peek_char() == Some('<') {
            self.consume_until('>')?;
            self.consume_char(); // consume '>'
            self.skip_whitespace_and_comments();
        }
        
        let name = self.consume_identifier().expect(format!("Failed to parse global B {declared_as:?}").as_str());
        
        self.skip_whitespace_and_comments();
        self.expect_char(':')?;
        self.skip_whitespace_and_comments();
        
        let ty = self.consume_type()?;
        
        self.skip_whitespace_and_comments();
        
        // Skip initialization if present
        if self.peek_char() == Some('=') {
            self.consume_until(';')?;
        }
        
        self.expect_char(';')?;
        
        Ok(ShaderElement::Global {
            attrs,
            declared_as,
            name,
            ty
        })
    }
    
    fn consume_type(&mut self) -> Result<String, Box<dyn Error>> {
        let mut ty = String::new();
        let mut depth = 0;
        
        loop {
            self.skip_whitespace();
            
            match self.peek_char() {
                Some('<') => {
                    ty.push(self.consume_char().unwrap());
                    depth += 1;
                }
                Some('>') => {
                    ty.push(self.consume_char().unwrap());
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                Some(',') if depth > 0 => {
                    ty.push(self.consume_char().unwrap());
                }
                Some(c) if c.is_alphanumeric() || c == '_' => {
                    ty.push(self.consume_char().unwrap());
                }
                _ => {
                    if depth == 0 {
                        break;
                    } else {
                        ty.push(self.consume_char().unwrap());
                    }
                }
            }
        }
        
        Ok(ty.trim().to_string())
    }
    
    fn consume_block(&mut self) -> Result<String, Box<dyn Error>> {
        self.expect_char('{')?;
        
        let mut block = String::from("{");
        let mut depth = 1;
        
        while depth > 0 {
            match self.consume_char() {
                Some('{') => {
                    depth += 1;
                    block.push('{');
                }
                Some('}') => {
                    depth -= 1;
                    block.push('}');
                }
                Some(c) => block.push(c),
                None => return Err(Box::new(ShaderPreProcessorError::ParseError(
                    "Unexpected end of input while parsing block".to_string()
                )))
            }
        }
        
        Ok(block)
    }
    
    fn consume_identifier(&mut self) -> Result<String, Box<dyn Error>> {
        let mut ident = String::new();
        
        while let Some(c) = self.peek_char() {
            if c == '<' {
                ident.push_str(&self.consume_until('>')?);
                ident.push(self.consume_char().unwrap());
            } else if c.is_alphanumeric() || c == '_' {
                ident.push(self.consume_char().unwrap());
            } else {
                break;
            }
        }
        
        if ident.is_empty() {
            Err(Box::new(ShaderPreProcessorError::ParseError(
                "Expected identifier".to_string()
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
            Err(Box::new(ShaderPreProcessorError::UnexpectedToken(
                format!("Expected '{}', got '{}'", expected, word)
            )))
        }
    }
    
    fn consume_until(&mut self, delim: char) -> Result<String, Box<dyn Error>> {
        let mut result = String::new();
        
        while let Some(c) = self.peek_char() {
            if c == delim {
                break;
            }
            result.push(self.consume_char().unwrap());
        }
        
        Ok(result)
    }
    
    fn expect_char(&mut self, expected: char) -> Result<(), Box<dyn Error>> {
        match self.consume_char() {
            Some(c) if c == expected => Ok(()),
            Some(c) => Err(Box::new(ShaderPreProcessorError::UnexpectedToken(
                format!("Expected '{}', got '{}'", expected, c)
            ))),
            None => Err(Box::new(ShaderPreProcessorError::ParseError(
                format!("Expected '{}', got end of input", expected)
            )))
        }
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
    
    fn peek_char(&self) -> Option<char> {
        self.input.chars().nth(self.pos)
    }
    
    fn consume_char(&mut self) -> Option<char> {
        let c = self.peek_char();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.consume_char();
            } else {
                break;
            }
        }
    }
    
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            self.skip_whitespace();
            
            if self.peek_char() == Some('/') && self.input.chars().nth(self.pos + 1) == Some('/') {
                // Skip line comment
                while self.peek_char().is_some() && self.peek_char() != Some('\n') {
                    self.consume_char();
                }
            } else if self.peek_char() == Some('/') && self.input.chars().nth(self.pos + 1) == Some('*') {
                // Skip block comment
                self.consume_char(); // '/'
                self.consume_char(); // '*'
                while !(self.peek_char() == Some('*') && self.input.chars().nth(self.pos + 1) == Some('/')) {
                    if self.consume_char().is_none() {
                        break;
                    }
                }
                self.consume_char(); // '*'
                self.consume_char(); // '/'
            } else {
                break;
            }
        }
    }
    
    fn is_at_end(&self) -> bool {
        self.pos >= self.input.len()
    }
}