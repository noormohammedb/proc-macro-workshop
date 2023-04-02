use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::parse_macro_input;
use syn::Result as ResultSyn;
use syn::Token;

#[derive(Debug)]
struct SeqMacroInput {/* ... */}

impl Parse for SeqMacroInput {
    fn parse(input: ParseStream) -> ResultSyn<Self> {
        let first_token = syn::Ident::parse(input);
        let _in = <Token![in]>::parse(input)?;
        let from = syn::LitInt::parse(input)?;
        let _dots = <Token![..]>::parse(input)?;
        let to = syn::LitInt::parse(input)?;
        let body = syn::Block::parse(input)?;

        Ok(SeqMacroInput {})
    }
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    // let _ = input;

    let ast = parse_macro_input!(input as SeqMacroInput);
    eprintln!("{:#?}", ast);

    // unimplemented!()
    TokenStream::new()
}
