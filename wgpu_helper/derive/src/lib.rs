// use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};
use syn::*;

mod bind_group_helpers;

#[proc_macro_derive(BindGroup, attributes(layout))]
pub fn derive_bind_group(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	bind_group_helpers::derive_bind_group(input)
}

#[proc_macro_derive(HostShareable, attributes())]
pub fn derive_host_shareable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let expanded = quote! {};

	proc_macro::TokenStream::from(expanded)
}
