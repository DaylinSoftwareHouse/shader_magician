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
    let field_idx = (0 .. fields.len() as u32).map(|a| LitInt::new(&a.to_string(), Span::call_site())).collect::<Vec<_>>();

    let inputs = quote! { (#(<#fields as magician_vgpu::bindable::BindGroupPart>::PartInput),*) };

    let entries = quote! {
        [
            #(
                #fields::layout_entry(vgpu, #field_idx, visibility)
            ),*
        ]
    };

    let groups = 
        if fields.len() == 0 { quote! { [] } }
        else if fields.len() == 1 {
            quote! {
                [
                    #(
                        #fields::group_entry(vgpu, #field_idx, input)
                    ),*
                ]
            }
        } else {
            quote! {
                [
                    #(
                        #fields::group_entry(vgpu, #field_idx, &input.#field_idx)
                    ),*
                ]
            }
        };

    TokenStream::from(quote! {
        impl magician_vgpu::BindGroupProvider for #name {
            type Input = #inputs;

            fn layout(
                vgpu: &magician_vgpu::VirtualGpu,
                visibility: wgpu::ShaderStages
            ) -> wgpu::BindGroupLayout {
                use magician_vgpu::bindable::BindGroupPart;
                vgpu.device().create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        label: None,
                        entries: &#entries
                    }
                )
            }

            fn group(
                vgpu: &magician_vgpu::VirtualGpu,
                layout: &wgpu::BindGroupLayout,
                input: &#inputs
            ) -> wgpu::BindGroup {
                use magician_vgpu::BindGroupPart;
                vgpu.device().create_bind_group(
                    &wgpu::BindGroupDescriptor {
                        layout: &layout,
                        label: None,
                        entries: &#groups
                    }
                )
            }
        }
    })
}

#[proc_macro_derive(UniformBufferObject)]
pub fn buffer_object(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);
    let name = item.ident.clone();
    
    TokenStream::from(quote! {
        impl magician_vgpu::bindable::BindGroupPart for #name {
            type PartInput = wgpu::Buffer;
            
            fn layout_entry(
                vgpu: &magician_vgpu::VirtualGpu,
                binding: u32, 
                visibility: wgpu::ShaderStages
            ) -> wgpu::BindGroupLayoutEntry {
                wgpu::BindGroupLayoutEntry {
                    binding, visibility,
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    }
                }
            }

            fn group_entry<'a>(
                vgpu: &'a magician_vgpu::VirtualGpu,
                binding: u32,
                input: &'a Self::PartInput
            ) -> wgpu::BindGroupEntry<'a> {
                use magician_vgpu::Buffer;
                wgpu::BindGroupEntry {
                    binding,
                    resource: input.as_entire_binding()
                }
            }
        }
    })
}
