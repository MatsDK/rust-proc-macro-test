use proc_macro::{self, TokenStream};
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream, Result},
    parse_macro_input, FnArg, Ident, Token,
};

struct Service {
    ident: Ident,
    methods: Vec<Method>
}

impl Parse for Service {
    fn parse(input: ParseStream) -> Result<Self> {
        <Token![trait]>::parse(input)?;
        let ident: Ident = input.parse()?;

        let content;
        braced!(content in input);

        let mut methods = Vec::new();
        while !content.is_empty() {
            methods.push(  content.parse::<Method>()?);
        }

        Ok(Service {
            ident,
            methods
        })
    }
}

#[derive(Debug)]
struct Method {
    ident: Ident
}

impl Parse for Method {
    fn parse(input: ParseStream) -> Result<Self> {
        <Token![async]>::parse(input)?;
        <Token![fn]>::parse(input)?;

        let ident: Ident = input.parse()?;

        let content;
        parenthesized!(content in input);

        <Token![;]>::parse(input)?;

        Ok(Method {
            ident
        })
    }
}

#[proc_macro_attribute]
pub fn service(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let Service { ident, methods } = parse_macro_input!(item as Service);
    println!("{:?}, {:?}", ident, methods);

    TokenStream::new()
}
