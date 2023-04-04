use proc_macro::TokenStream;
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
    let _braces = braced!(content in input);
    let tt = proc_macro2::TokenStream::parse(&content)?;

    Ok(SeqMacroInput {
      from,
      to,
      ident,
      tt,
    })
  }
}

impl Into<proc_macro2::TokenStream> for SeqMacroInput {
  fn into(self) -> proc_macro2::TokenStream {
    (self.from..self.to)
      .map(|i| self.expand(self.tt.clone(), i))
      .collect()
  }
}

impl SeqMacroInput {
  fn expand2(
    &self,
    tt: proc_macro2::TokenTree,
    rest: &mut proc_macro2::token_stream::IntoIter,
    i: isize,
  ) -> proc_macro2::TokenTree {
    match tt {
      proc_macro2::TokenTree::Group(g) => {
        let mut expanded = proc_macro2::Group::new(g.delimiter(), self.expand(g.stream(), i));
        expanded.set_span(g.span());
        proc_macro2::TokenTree::Group(expanded)
      }

      proc_macro2::TokenTree::Ident(ident) if ident == self.ident => {
        let mut lit = proc_macro2::Literal::isize_unsuffixed(i);
        lit.set_span(ident.span());
        proc_macro2::TokenTree::Literal(lit)
      }

      proc_macro2::TokenTree::Ident(mut ident) => {
        let mut peek = rest.clone();
        match (peek.next(), peek.next()) {
          (
            Some(proc_macro2::TokenTree::Punct(ref pnc)),
            Some(proc_macro2::TokenTree::Ident(ref ident2)),
          ) if pnc.as_char() == '~' && ident2 == &self.ident => {
            ident = proc_macro2::Ident::new(&format!("{}{}", ident, i), ident.span());
            *rest = peek.clone();
            match peek.next() {
              Some(proc_macro2::TokenTree::Punct(ref pnc2)) if pnc2.as_char() == '~' => {
                *rest = peek.clone();
              }
              _ => {}
            }
          }
          _ => {}
        }
        proc_macro2::TokenTree::Ident(ident)
      }
      tt => tt,
    }
  }

  fn expand(&self, stream: proc_macro2::TokenStream, i: isize) -> proc_macro2::TokenStream {
    let mut out = proc_macro2::TokenStream::new();
    let mut tts = stream.into_iter();
    while let Some(tt) = tts.next() {
      out.extend(std::iter::once(self.expand2(tt, &mut tts, i)));
    }
    out
  }
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as SeqMacroInput);
  let output: proc_macro2::TokenStream = input.into();
  output.into()
}
