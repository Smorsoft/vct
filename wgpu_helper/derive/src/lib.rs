// use proc_macro2::TokenStream;
use quote::quote;
use syn::*;

mod bind_group_helpers;

#[proc_macro_derive(BindGroup, attributes())]
pub fn derive_bind_group(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let name = input.ident;
	let generics = input.generics;
	let visibility = input.vis;

	let layout_descriptor_name = syn::Ident::new(&format!("AUTO_GENERATED_{}_LAYOUT", &name), proc_macro2::Span::call_site());
	let newtype_bind_group_name = syn::Ident::new(&format!("{}NT", &name), proc_macro2::Span::call_site());

	let source_type_generics = bind_group_helpers::parse_generics_for_source_type(&generics);

	let mut layout_entries = proc_macro2::TokenStream::new();
	let mut bind_group_entries = proc_macro2::TokenStream::new();

	match input.data {
		Data::Struct(ref data) => match data.fields {
			Fields::Named(ref fields) => {
				let recurse = fields.named.iter().enumerate().map(|(i, f)| {
					quote! {}
				});
			}
			Fields::Unit => panic!("Unit structs are not supported"),
			_ => panic!("Unnamed fields are not supported"),
		},
		Data::Enum(_) | Data::Union(_) => panic!("Enums and Unions are unsupported as bind groups"),
	};

	let expanded = quote! {
		static #layout_descriptor_name: ::std::sync::OnceLock<::wgpu::BindGroupLayout> = ::std::sync::OnceLock::new();

		#[repr(transparent)]
		#visibility struct #newtype_bind_group_name(::wgpu::BindGroup);

		impl ::wgpu_helper::bind_group::BindGroup for #newtype_bind_group_name {
			type Source = #name<#source_type_generics>;
		}

		impl #generics ::wgpu_helper::bind_group::BindGroupType for #name #generics {
			type BindGroup = #newtype_bind_group_name;

			const BIND_GROUP_LAYOUT_DESCRIPTOR: &'static ::wgpu::BindGroupLayoutDescriptor<'static> = &::wgpu::BindGroupLayoutDescriptor {
				label: Some("#name Layout Descriptor"),
				entries: &[
					#layout_entries
				],
			};

			fn get_bind_group_lock() -> &'static ::std::sync::OnceLock<::wgpu::BindGroupLayout> {
				&#layout_descriptor_name
			}

			fn to_bind_group(&self, device: &wgpu::Device, label: Option<&str>) -> Self::BindGroup {
				unsafe {
					Self::BindGroup::from_untyped(device.create_bind_group(&wgpu::BindGroupDescriptor {
						label,
						layout: Self::get_bind_group_layout(device),
						entries: &[
							#bind_group_entries
						],
					}))
				}
			}
		}

		// impl #name {

		// }
	};

	proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(HostShareable, attributes())]
pub fn derive_host_shareable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let expanded = quote! {};

	proc_macro::TokenStream::from(expanded)
}
