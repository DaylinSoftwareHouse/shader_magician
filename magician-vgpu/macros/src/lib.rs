use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStruct, parse_macro_input};

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
            quote! { &'a dyn magician_vgpu::buffers::Buffer<Type = #field_ty> }
        } else { quote! { (#(&'a dyn magician_vgpu::buffers::Buffer<Type = #fields>),*) } };

    let layout_indices = fields.iter().enumerate().map(|(a, _)| a as u32).collect::<Vec<_>>();
    let layout_entries = quote! {
        [
            #(
                wgpu::BindGroupLayoutEntry {
                    binding: #layout_indices,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    }
                }
            ),*
        ]
    };

    let layout_bindings = if fields.is_empty() { quote!{ [] } }
        else if fields.len() == 1 { 
            quote! {
                [
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: inputs.buffer().as_entire_binding()
                    }
                ]
            }
        } else {
            quote! {
                [
                    #(
                        wgpu::BindGroupEntry {
                            binding: #layout_indices,
                            resource: inputs.#layout_indices.buffer().as_entire_binding()
                        }
                    ),*
                ]
            }
        };

    TokenStream::from(quote! {
        impl <'a> magician_vgpu::bindable::BindableObjectCreator<'a> for #name {
            type Inputs = #inputs;

            fn create_object(
                vgpu: &magician_vgpu::VirtualGpu, 
                inputs: Self::Inputs
            ) -> magician_vgpu::BindableObject<Self> where Self: Sized
            {
                let layout = vgpu.device().create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        label: None,
                        entries: &#layout_entries
                    }
                );

                let bind_group = vgpu.device().create_bind_group(
                    &wgpu::BindGroupDescriptor {
                        layout: &layout,
                        label: None,
                        entries: &#layout_bindings
                    }
                );

                magician_vgpu::bindable::BindableObject::new(bind_group, layout)
            }
        }
    })
}
