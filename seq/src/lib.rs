use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::Result as ResultSyn;
use syn::{braced, parse_macro_input, Token};

#[derive(Debug, Clone)]
struct SeqMacroInput {
  from: isize,
  to: isize,
  ident: syn::Ident,
  tt: proc_macro2::TokenStream,
}

impl Parse for SeqMacroInput {
  fn parse(input: ParseStream) -> ResultSyn<Self> {
    let ident = syn::Ident::parse(input)?;
    let _in = <Token![in]>::parse(input)?;
    let from = syn::LitInt::parse(input)?.base10_parse()?;
    let _dots = <Token![..]>::parse(input)?;
    let to = syn::LitInt::parse(input)?.base10_parse()?;
    let content;
    let braces = braced!(content in input);
    let tt = proc_macro2::TokenStream::parse(&content)?;

    Ok(SeqMacroInput {
      from,
      to,
      ident,
      tt,
    })
  }
}

impl Into<TokenStream> for SeqMacroInput {
  fn into(self) -> TokenStream {
    let mut out = proc_macro2::TokenStream::new();

    for i in self.from..self.to {
      let expanded = self.expand(self.clone().tt, i);

      out = quote! { #out #expanded };
    }
    out.into()
  }
}

impl SeqMacroInput {
  fn expand2(&self, tt: proc_macro2::TokenTree, i: isize) -> proc_macro2::TokenTree {
    match tt {
      proc_macro2::TokenTree::Group(g) => {
        let mut expanded = proc_macro2::Group::new(g.delimiter(), self.expand(g.stream(), i));
        expanded.set_span(g.span());
        proc_macro2::TokenTree::Group(expanded).into()
      }
      proc_macro2::TokenTree::Ident(i) if i == self.ident => {
        /*

        */
        // proc_macro2::TokenTree::Literal(syn::Lit::from(i)).into()

        let mut lit = proc_macro2::Literal::isize_suffixed(i.to_string().parse().unwrap());

        proc_macro2::TokenTree::Literal(lit).into()
      }

      _ => tt,
    }
  }

  fn expand(&self, stream: proc_macro2::TokenStream, i: isize) -> proc_macro2::TokenStream {
    stream
      .into_iter()
      .map(|tt| self.expand2(tt.into(), i))
      .collect()
  }
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as SeqMacroInput);

  input.into()
}
