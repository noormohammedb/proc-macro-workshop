extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    // eprintln!("{:#?}", ast);
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

    // let fields = match ast.data {
    //     syn::Data::Struct(syn::DataStruct {
    //         fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
    //         ..
    //     }) => named,
    //     _ => unimplemented!(),
    // };

    let ty_is_option = |f: &syn::Field| {
        if let syn::Type::Path(p) = &f.ty {
            return p.path.segments.len() == 1 && p.path.segments[0].ident == "Option";
        }
        false
    };

    let optionized = fields.iter().map(|fld| {
        let name = &fld.ident;
        let ty = &fld.ty;
        if ty_is_option(&fld) {
            quote! { #name: #ty }
        } else {
            quote! { #name: std::option::Option<#ty> }
        }
    });

    let methods = fields.iter().map(|fld| {
        let name = &fld.ident;
        let ty = &fld.ty;
        if ty_is_option(&fld) {
            quote! {
                fn #name(&mut self, #name: #ty) -> &mut Self{
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
        }
    });

    let build_fields = fields.iter().map(|fld| {
        let name = &fld.ident;
        if ty_is_option(fld) {
            quote! {
                #name: self.#name.clone()
            }
        } else {
            quote! {
                #name: self.#name.clone().ok_or(concat!(stringify!(#name), "is not set"))?
            }
        }
    });

    let build_empty = fields.iter().map(|fld| {
        let name = &fld.ident;
        quote! { #name: None }
    });

    quote!(
        pub struct #builder_ident {
            #(#optionized,)*
        }
        impl #builder_ident {
            #(#methods)*
            pub fn build(&mut self) -> Result<Command, Box<dyn std::error::Error>> {
                Ok(#name {
                    #(#build_fields,)*
                })

            }
        }
        impl #name {
            pub  fn builder() -> #builder_ident {
                #builder_ident {
                    #(#build_empty,)*
                }
            }
        }
    )
    .into()
}
