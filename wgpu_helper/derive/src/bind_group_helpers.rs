use quote::{quote, ToTokens, TokenStreamExt};
use syn::spanned::Spanned;
use syn::*;

use syn::punctuated::Punctuated;

pub fn derive_bind_group(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let name = input.ident;
	let generics = input.generics;
	let visibility = input.vis;

	let layout_descriptor_name = syn::Ident::new(
		&format!("AUTO_GENERATED_{}_LAYOUT", &name),
		proc_macro2::Span::call_site(),
	);
	let newtype_bind_group_name =
		syn::Ident::new(&format!("{}NT", &name), proc_macro2::Span::call_site());

	let source_type_generics = parse_generics_for_source_type(&generics);

	// let mut layout_entries = proc_macro2::TokenStream::new();
	let mut bind_group_entries = proc_macro2::TokenStream::new();

	let mut bind_group_layout = quote! {};

	for attr in input.attrs {
		if attr.path().is_ident("layout") {
			match attr.meta {
				Meta::List(meta) => {
					bind_group_layout = meta.tokens;
				} 
				_ => panic!("Unsupported format of layout attribute")
			}
		}
	}

	match input.data {
		Data::Struct(ref data) => match data.fields {
			Fields::Named(ref fields) => {
				for (i, f) in fields.named.iter().enumerate() {
					let field_name = f.ident.to_owned().expect("Unnamed fields are not allowed");
					let binding = i as u32;

					// layout_entries.append_all(parse_attributes_for_layout(binding, f));

					let resource_binding: proc_macro2::TokenStream = format!(
						"::wgpu_helper::BindGroupItem::get_binding(self.{})",
						field_name
					)
					.parse()
					.unwrap();

					bind_group_entries.append_all(quote!(
						::wgpu::BindGroupEntry {
							binding: #binding,
							resource: #resource_binding,
						},
					));
				}
			}
			Fields::Unit => panic!("Unit structs are not supported"),
			Fields::Unnamed(_) => panic!("Unnamed fields are not supported"),
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

			const BIND_GROUP_LAYOUT_DESCRIPTOR: &'static ::wgpu::BindGroupLayoutDescriptor<'static> = &#bind_group_layout;

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
	};

	proc_macro::TokenStream::from(expanded)
}

pub fn parse_attributes_for_layout(binding: u32, field: &syn::Field) -> proc_macro2::TokenStream {
	let mut visibility = None;
	let mut ty = quote! {};
	let mut count = quote! {::core::option::Option::None};

	let mut source_type = quote! {};

	for attr in field.attrs.iter() {
		if attr.path().is_ident("layout") {
			let nested = attr
				.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
				.expect("Invalid format of attribute layout");
			for meta in nested {
				match meta {
					Meta::Path(_) => {}
					Meta::List(meta) => {
						if meta.path.is_ident("ty") {
							let nested = meta
								.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
								.expect("Invalid format of binding type");
							for meta in nested {
								match meta {
									Meta::Path(_) => {}
									Meta::List(meta) => {
										if meta.path.is_ident("buffer") {
											// let mut 
											let mut has_dynamic_offset = false;

											let nested = meta
												.parse_args_with(
													Punctuated::<Meta, Token![,]>::parse_terminated,
												)
												.expect("Invalid format of buffer binding type");

											for meta in nested {
												match meta {
													Meta::NameValue(_) => {},
													Meta::Path(meta) => {
														meta.segments.last().unwrap().ident.to_token_stream();
														
													},
													Meta::List(meta) => {

													},
												}
											}

											ty = quote! {
												::wgpu::BindingType::Buffer {
													ty: ,
													has_dynamic_offset: #has_dynamic_offset,
													min_binding_size: ::core::option::Option::Some(::core::num::NonZeroU32(#source_type)),
												}
											}
										}
									}
									Meta::NameValue(meta) => {
										if meta.path.is_ident("acceleration_structure") {
											ty = quote! {::wgpu::BindingType::AccelerationStructure}
										}
									}
								}
							}
						}
					}
					Meta::NameValue(meta) => {
						if meta.path.is_ident("visibility") {
							let mut path = Path {
								leading_colon: Some(Token![::](meta.span())),
								segments: Punctuated::new(),
							};

							path.segments.push(PathSegment {
								ident: Ident::new("wgpu", meta.span()),
								arguments: PathArguments::None,
							});

							path.segments.push(PathSegment {
								ident: Ident::new("ShaderStages", meta.span()),
								arguments: PathArguments::None,
							});

							match meta
								.value
								.to_token_stream()
								.to_string()
								.to_lowercase()
								.as_str()
							{
								"none" => path.segments.push(PathSegment {
									ident: Ident::new("NONE", meta.span()),
									arguments: PathArguments::None,
								}),
								"vertex" => path.segments.push(PathSegment {
									ident: Ident::new("VERTEX", meta.span()),
									arguments: PathArguments::None,
								}),
								"compute" => path.segments.push(PathSegment {
									ident: Ident::new("COMPUTE", meta.span()),
									arguments: PathArguments::None,
								}),
								"fragment" => path.segments.push(PathSegment {
									ident: Ident::new("FRAGMENT", meta.span()),
									arguments: PathArguments::None,
								}),
								"vertex_fragment" => path.segments.push(PathSegment {
									ident: Ident::new("VERTEX_FRAGMENT", meta.span()),
									arguments: PathArguments::None,
								}),
								_ => panic!("A unsupported visibility was given"),
							}

							visibility = Some(path);
						}

						if meta.path.is_ident("count") {
							let expr_text = meta.value.to_token_stream().to_string().to_lowercase();

							if expr_text.as_str() == "none" {
								count = quote! {::core::option::Option::None};
							} else {
								let num: u32 = expr_text
									.parse()
									.expect("Count should either be a number or None");
								count = quote! {::core::option::Option::Some(::core::num::NonZeroU32::new( #num ))};
							}
						}

						if meta.path.is_ident("source_type") {
							source_type = meta.value.to_token_stream();
						}
					}
				}
			}
		}
	}

	quote! {
		::wgpu::BindGroupLayoutEntry {
			binding: #binding,
			visibility: #visibility,
			ty: #ty,
			count: #count,
		},
	}
}

pub fn parse_generics_for_source_type(generics: &Generics) -> Option<syn::Lifetime> {
	if let Some(generic_param) = generics.params.first() {
		// https://docs.rs/syn/latest/syn/enum.GenericParam.html
		match generic_param {
			syn::GenericParam::Lifetime(_) => Some(syn::Lifetime::new(
				"'static",
				proc_macro2::Span::call_site(),
			)),
			syn::GenericParam::Const(_) => unimplemented!(),
			syn::GenericParam::Type(_) => unimplemented!(),
		}
	} else {
		None
	}
}
