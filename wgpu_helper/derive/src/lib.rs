// use proc_macro2::TokenStream;
use quote::quote;
use syn::*;

#[proc_macro_derive(BindGroup, attributes())]
pub fn derive_bind_group(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let name = input.ident;

	let entries = match input.data {
		Data::Struct(ref data) => {
			match data.fields {
				Fields::Named(ref fields) => {
					let recurse = fields.named.iter().enumerate().map(|(i, f)| {
						quote! {}
					});
					quote! {}
				},
				Fields::Unit => {
					quote! {

					}
				}
				_ => panic!("Unnamed fields are not supported"),
			}
		},
		Data::Enum(_) | Data::Union(_) => panic!("Enums and Unions are unsupported as bind groups")
	};
	
	let expanded = quote! {
		impl ::wgpu_helper::bind_group::BindGroup for #name {
			fn get_bind_group_layout_descriptor() -> ::wgpu::BindGroupLayoutDescriptor<'static> {

			}
		}

		impl #name {
			
		}
	};
	
	proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(HostShareable, attributes())]
pub fn derive_wgsl_struct(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let expanded = quote! {};

	proc_macro::TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {
	#[derive(HostShareable)]
	struct Test {
		field1: String,
	}
}
