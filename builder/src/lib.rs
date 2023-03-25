extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    eprintln!("{:#?}", ast);
    let name = &ast.ident;
    let builder_name = format!("{}Builder", name);
    let builder_ident = syn::Ident::new(&builder_name, name.span());
    let expanded = quote! {
    struct #builder_ident {
    }
    impl #name {
         fn builder() -> #builder_ident {
            #builder_ident{
            }
        }
    }};
    expanded.into()
}
