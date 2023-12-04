use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(BindGroup, attributes())]
pub fn derive_bind_group(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let expanded = quote! {};

	TokenStream::from(expanded)
}

#[proc_macro_derive(HostShareable, attributes())]
pub fn derive_wgsl_struct(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let expanded = quote! {};

	TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {
	#[derive(HostShareable)]
	struct Test {
		field1: String,
	}
}
