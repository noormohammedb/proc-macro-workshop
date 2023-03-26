extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let builder_name = format!("{}Builder", name);
    let builder_ident = syn::Ident::new(&builder_name, name.span());

    quote!(
        pub struct #builder_ident {
            executable: Option<String>,
            args: Option<Vec<String>>,
            env: Option<Vec<String>>,
            current_dir: Option<String>,
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
                    executable: self.executable.clone().ok_or("executable not found")?,
                    args: self.args.clone().ok_or("args not found")?,
                    env: self.env.clone().ok_or("env not found")?,
                    current_dir: self.current_dir.clone().ok_or("current_dir not found")?,
                })

            }
        }
        impl #name {
            pub  fn builder() -> #builder_ident {
                #builder_ident {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
        }
    )
    .into()
}
