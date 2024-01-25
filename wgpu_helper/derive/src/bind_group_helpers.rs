use syn::Generics;

pub fn parse_generics_for_source_type(generics: &Generics) -> Option<syn::Lifetime> {
	if let Some(generic_param) = generics.params.first() {
		// https://docs.rs/syn/latest/syn/enum.GenericParam.html
		match generic_param {
			syn::GenericParam::Type(_) => unimplemented!(),
			syn::GenericParam::Lifetime(_) => Some(syn::Lifetime::new("'static", proc_macro2::Span::call_site())),
			syn::GenericParam::Const(_) => unimplemented!(),
		}
	} else {
		None
	}
}
