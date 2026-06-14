use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{ItemStruct, LitInt, parse_macro_input};

#[proc_macro_derive(BindableObject, attributes(uniform, read, write))]
pub fn bindable_object(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);
    let name = item.ident.clone();
    let fields = item.fields.iter()
        .map(|field| field.ty.clone())
        .collect::<Vec<_>>();

    let inputs = 
        if fields.len() <= 0 { quote! { () } }
        else if fields.len() == 1 {
            let field_ty = &fields[0];
            quote! { dyn magician_vgpu::Buffer<Type = #field_ty> }
        } else { quote! { (#(dyn magician_vgpu::Buffer<Type = #fields>),*) } };

    TokenStream::from(quote! {
        
    })
}

#[proc_macro_derive(BufferObject)]
pub fn buffer_object(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);
    let name = item.ident.clone();
    
    TokenStream::from(quote! {
        impl <B: magician_vgpu::Buffer<Type = #name>> magician_vgpu::bindable::BindGroupPart<B> for #name {
            fn layout_entry(
                vgpu: &magician_vgpu::VirtualGpu,
                binding: u32, 
                visibility: wgpu::ShaderStages
            ) -> wgpu::BindGroupLayoutEntry {
                B::layout_entry(vgpu, binding, visibility)
            }

            fn group_entry<'a>(
                vgpu: &'a magician_vgpu::VirtualGpu,
                binding: u32,
                input: &'a B
            ) -> wgpu::BindGroupEntry<'a> {
                B::group_entry(vgpu, binding, input)
            }
        }
    })
}
