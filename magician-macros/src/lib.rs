use proc_macro::TokenStream;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_attribute]
pub fn shader(_attr: TokenStream, item: TokenStream) -> TokenStream {
    return item;
}

#[proc_macro_derive(ShaderLayout, attributes(location, builtin))]
pub fn shader_layout(input: TokenStream) -> TokenStream {
    // decode input
    let input = parse_macro_input!(input as DeriveInput);
    let _struct_name = &input.ident;

    // get only named fields, otherwise, panic
    let _fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("ShaderLayout only supports named fields")
        },
        _ => panic!("ShaderLayout only supports structs")
    };

    TokenStream::new()
}

#[proc_macro_derive(ShaderGroup, attributes(uniform, read, write))]
pub fn shader_group(input: TokenStream) -> TokenStream {
    // decode input
    let input = parse_macro_input!(input as DeriveInput);
    let _struct_name = &input.ident;

    // get only named fields, otherwise, panic
    let _fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("ShaderLayout only supports named fields")
        },
        _ => panic!("ShaderLayout only supports structs")
    };

    TokenStream::new()
}
