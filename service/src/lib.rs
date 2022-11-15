use proc_macro::{self, TokenStream};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    token::Comma,
    FnArg, Ident, PatType, ReturnType, Token,
};

struct Service {
    ident: Ident,
    methods: Vec<Method>,
}

struct Method {
    ident: Ident,
    output: ReturnType,
    args: Vec<PatType>,
}

impl Parse for Service {
    fn parse(input: ParseStream) -> Result<Self> {
        // input.parse::<Token![trait]>()?;
        <Token![trait]>::parse(input)?;
        let ident: Ident = input.parse()?;

        let content;
        braced!(content in input);

        let mut methods = Vec::new();
        while !content.is_empty() {
            methods.push(<Method>::parse(&content)?);
        }

        Ok(Service { ident, methods })
    }
}

impl Parse for Method {
    fn parse(input: ParseStream) -> Result<Self> {
        <Token![async]>::parse(input)?;
        <Token![fn]>::parse(input)?;

        let ident: Ident = input.parse()?;

        let content;
        parenthesized!(content in input);

        let mut args = Vec::new();
        for arg in content.parse_terminated::<FnArg, Comma>(FnArg::parse)? {
            match arg {
                FnArg::Typed(p) => args.push(p),
                FnArg::Receiver(_) => {
                    eprintln!("Not supported")
                }
            }
        }

        let output = input.parse()?;
        <Token![;]>::parse(input)?;

        Ok(Method {
            ident,
            output,
            args,
        })
    }
}

#[proc_macro_attribute]
pub fn service(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let Service { ident, methods } = parse_macro_input!(item as Service);

    ServiceGenerator {
        service_ident: ident,
        methods,
    }
    .into_token_stream()
    .into()
}

struct ServiceGenerator {
    service_ident: Ident,
    methods: Vec<Method>,
}

impl ServiceGenerator {
    fn service_trait(&self) -> TokenStream2 {
        let ServiceGenerator {
            service_ident,
            methods,
            ..
        } = self;

        let types_and_fns = methods.iter().map(|Method { ident, output, .. }| {
            quote! {
                fn #ident(self, ctx: String) #output;
            }
        });

        quote! {
            trait #service_ident {
                #( #types_and_fns )*
            }
        }
    }

    fn server_struct(&self) -> TokenStream2 {
        let ServiceGenerator { service_ident, .. } = self;

        let server_ident = format_ident!("{}Server", service_ident);

        quote! {
            struct #server_ident {

            }
        }
    }
}

impl ToTokens for ServiceGenerator {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend(vec![self.service_trait(), self.server_struct()])
    }
}
