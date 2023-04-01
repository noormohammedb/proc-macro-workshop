extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let builder_name = format!("{}Builder", name);
    let builder_ident = syn::Ident::new(&builder_name, name.span());

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = ast.data
    {
        named
    } else {
        unimplemented!();
    };

    let optionized = fields.iter().map(|fld| {
        let name = &fld.ident;
        let ty = &fld.ty;
        if ty_inner_type("Option", &ty).is_some() || builder_of(fld).is_some() {
            quote! { #name: #ty }
        } else {
            quote! { #name: std::option::Option<#ty> }
        }
    });

    let methods_builder = fields.iter().map(|fld| {
        let name = &fld.ident;
        let ty = &fld.ty;

        let set_method = if let Some(inner_ty) = ty_inner_type("Option", &ty) {
            quote! {
                fn #name(&mut self, #name: #inner_ty) -> &mut Self{
                    self.#name = Some(#name);
                    self
                }
            }
        } else if builder_of(&fld).is_some() {
            quote! {
                pub fn #name(&mut self, #name: #ty) -> &mut Self {
                    self.#name = #name;
                    self
                }
            }
        } else {
            quote! {
                fn #name(&mut self, #name: #ty) -> &mut Self{
                    self.#name = Some(#name);
                    self
                }
            }
        };

        match extend_methods_builder(&fld) {
            None => set_method,
            Some((true, extended_method)) => extended_method,
            Some((false, extended_method)) => {
                quote! {
                    #set_method
                    #extended_method
                }
            }
        }
    });

    let build_fields = fields.iter().map(|fld| {
        let name = &fld.ident;
        let ty = &fld.ty;
        if ty_inner_type("Option", &ty).is_some() || builder_of(fld).is_some() {
            quote! {
                #name: self.#name.clone()
            }
        } else {
            quote! {
                #name: self.#name.clone().ok_or(concat!(stringify!(#name), " is not set"))?
            }
        }
    });

    let build_empty = fields.iter().map(|fld| {
        let name = &fld.ident;
        if builder_of(fld).is_some() {
            return quote! { #name: Vec::new() };
        } else {
            return quote! { #name: None };
        }
    });

    quote!(
        pub struct #builder_ident {
            #(#optionized,)*
        }
        impl #builder_ident {
            #(#methods_builder)*
            pub fn build(&mut self) -> Result<Command, Box<dyn std::error::Error>> {
                Ok(#name {
                    #(#build_fields,)*
                })

            }
        }
        impl #name {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    #(#build_empty,)*
                }
            }
        }
    )
    .into()
}

fn ty_inner_type<'a>(wrapper: &'a str, ty: &'a syn::Type) -> Option<&'a syn::Type> {
    if let syn::Type::Path(ref p) = ty {
        if p.path.segments.len() != 1 || p.path.segments[0].ident != wrapper {
            return None;
        }
        if let syn::PathArguments::AngleBracketed(ref inner_ty) = p.path.segments[0].arguments {
            if inner_ty.args.len() != 1 {
                return None;
            }

            let inner_ty = inner_ty.args.first().unwrap();

            if let syn::GenericArgument::Type(t) = inner_ty {
                return Some(t);
            }
        }
    }
    None
}

fn builder_of(fld: &syn::Field) -> Option<&syn::MetaList> {
    for atter in &fld.attrs {
        let name = &fld.ident;
        let segments = &atter.path().segments;
        if segments.len() == 1 && segments[0].ident == "builder" {
            if let syn::Meta::List(meta_list) = &atter.meta {
                return Some(meta_list);
            }
        }
    }
    None
}

fn extend_methods_builder(fld: &syn::Field) -> Option<(bool, proc_macro2::TokenStream)> {
    let meta_list = builder_of(fld)?;
    let mut tokens = meta_list.tokens.clone().into_iter();
    match tokens.next().unwrap() {
        TokenTree::Ident(ref ide) => {
            if ide != "each" {
                return Some((
                    false,
                    syn::Error::new_spanned(
                        &fld.attrs[0].meta,
                        "expected `builder(each = \"...\")`",
                    )
                    .to_compile_error(),
                ));
            }
        }
        tt => panic!("expected 'each', found {}", tt),
    }
    match tokens.next().unwrap() {
        TokenTree::Punct(ref pnc) => {
            if pnc.as_char() != '=' {
                return Some((
                    false,
                    syn::Error::new_spanned(pnc, format!("expected '=' found {}", pnc.as_char()))
                        .to_compile_error(),
                ));
            }
        }
        tt => panic!("expected '=', found {}", tt),
    }
    let last_token = meta_list.tokens.clone().into_iter().nth(2).unwrap();
    let literal_ident = match last_token {
        TokenTree::Literal(lit) => lit,
        tt => panic!("expected string, found {}", tt),
    };
    match syn::Lit::new(literal_ident) {
        syn::Lit::Str(ident_str) => {
            let method_ident = syn::Ident::new(&ident_str.value(), ident_str.span());
            let inner_ty = ty_inner_type("Vec", &fld.ty);
            let name = &fld.ident;
            let method = quote! {
                pub fn #method_ident(&mut self, #method_ident: #inner_ty) -> &mut Self {
                    self.#name.push(#method_ident);
                    self
                }
            };
            return Some((method_ident == fld.ident.clone().unwrap(), method));
        }
        lit => panic!("expected string, found {:?}", lit),
    }
}
