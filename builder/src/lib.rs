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

    let fields = match &ast.data {
        syn::Data::Struct(syn::DataStruct { fields, .. }) => match fields {
            syn::Fields::Named(syn::FieldsNamed { named, .. }) => named,
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    // let named_fields = match ast.data {
    //     syn::Data::Struct(syn::DataStruct {
    //         fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
    //         ..
    //     }) => named,
    //     _ => todo!(),
    // };

    // let fields = match ast.data {
    //     syn::Data::Struct(syn::DataStruct { fields, .. }) => match fields {
    //         syn::Fields::Named(syn::FieldsNamed { ref named, .. }) => named,
    //         _ => unimplemented!(),
    //     },
    //     _ => unimplemented!(),
    // };

    let fields_iter = fields.iter();

    let optionized = fields.iter().map(|fld| {
        let name = &fld.ident;
        let ty = &fld.ty;
        quote! { #name: std::option::Option<#ty> }
    });

    let methods = fields.iter().map(|fld| {
        let name = &fld.ident;
        let ty = &fld.ty;
        quote! {
            fn #name(&mut self, #name: #ty) -> &mut Self{
                self.#name = Some(#name);
                self
            }
        }
    });

    let build_fields = fields.iter().map(|fld| {
        let name = &fld.ident;
        quote! {
            #name: self.#name.clone().ok_or(concat!(stringify!(#name), "is not set"))?
        }
    });

    let build_empty = fields.iter().map(|fld| {
        let name = &fld.ident;
        quote! { #name: None }
    });

    quote!(
        pub struct #builder_ident {
            // #(#fields_iter,)*
            #(#optionized,)*
        }
        impl #builder_ident {
            #(#methods)*

            pub fn build(&self) -> Result<Command, Box<dyn std::error::Error>> {
                Ok(#name {
                    #(#build_fields,)*

                    // executable: self.executable.clone().ok_or("executable not found")?,
                    // args: self.args.clone().ok_or("args not found")?,
                    // env: self.env.clone().ok_or("env not found")?,
                    // current_dir: self.current_dir.clone().ok_or("current_dir not found")?,
                })

            }
        }
        impl #name {
            pub  fn builder() -> #builder_ident {
                #builder_ident {
                    #(#build_empty,)*
                    // executable: None,
                    // args: None,
                    // env: None,
                    // current_dir: None,
                }
            }
        }
    )
    .into()
}
