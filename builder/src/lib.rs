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
        // let t = ;
        // let t = match fld.ty {
        //     syn::Type::Path(
        //         TypePath{syn::Path{
        //             segments
        //         }}
        //     )}  => _
        //     _ => _
        // };

        let type_segment = match &fld.ty {
            syn::Type::Path(typePath) => match typePath {
                // syn::Path { segments, .. } => segments[0],
                syn::TypePath { path, .. } => match path {
                    syn::Path { segments, .. } => segments,
                    _ => unimplemented!(),
                },
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        };

        let foo = type_segment.iter().last();
        // let bar = foo?.ident.ident;

        if type_segment.len() == 1 && type_segment.iter().last().unwrap().ident == "Option" {
            let expr = quote! {
                #name: self.#name.clone()
            };
            return expr;
        }

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
            fn executable(&mut self, executable: String) -> &mut Self {
                self.executable = Some(executable);
                self
            }
            fn args(&mut self, arg: Vec<String>) -> &mut Self {
                self.args = Some(arg);
                self
            }
            fn env(&mut self, envs: Vec<String>) -> &mut Self {
                self.env = Some(envs);
                self
            }
            fn current_dir(&mut self, dir: String) -> &mut Self {
                self.current_dir = Some(dir);
                self
            }
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
