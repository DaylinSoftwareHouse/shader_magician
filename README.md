# Magician
A tool for making all things WGPU easier to work with in Rust.  This project 
provides three main tools: a syn-like parsing library (magician-ast), a Rust
to WGSL converter macro for writing shaders in Rust (magician-rust), and a 
`VirtualGpu` implementation that simplifies common operations with the GPU to 
significantly reduce the boilerplate and complexity that comes with WGPU.

## Magician-AST
This crate serves as a simple syn-like library for parsing and serializing WGSL
shader code.  This allows for in Rust manipulation of shader code, more complex 
compilation steps, and was significantly helpful in making a transpiler for converting 
Rust to WGSL.

## Magician-Rust
This crate allow for the conversion rust functions to WGSL shader code via the
`#[shader("<path-to-shader-output-folder>")]` macro.  This will however, require
the addition of this code to your projects `build.rs` file for the macros to
be automatically converted.

```rust
magician_rust::build(
    "src".into(),
    "shader_out".into(),
    Some("shader_dbg".into()) // or None if you dont care about internal magician debugging :(
);
```

### Limitations
Hard Limits (no method to get around found):
 - No macros

Soft Limits (fix known, awaiting completion):
 - No enums yet
 - No tuples yet

## Magician VGPU
This work in progress module allows for simpler interactions with WGPU to reduce
boilerplate significantly while simplifying the process of doing anything GPU.