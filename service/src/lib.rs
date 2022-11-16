use convert_case::{Case, Casing};
use proc_macro::{self, TokenStream};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::{
    braced,
    ext::IdentExt,
    parenthesized,
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
    let Service {
        ref ident,
        ref methods,
    } = parse_macro_input!(item as Service);

    let method_names: &Vec<_> = &methods
        .iter()
        .map(|method| snake_to_camel_case(&method.ident.unraw().to_string()))
        .collect();

    ServiceGenerator {
        service_ident: &ident,
        methods,
        server_ident: &format_ident!("{}Server", ident),
        methods_enum_ident: &format_ident!("{}Methods", ident),
        method_idents: &methods
            .iter()
            .zip(method_names.iter())
            .map(|(method, name)| Ident::new(&name.to_string(), method.ident.span()))
            .collect::<Vec<_>>(),
    }
    .into_token_stream()
    .into()
}

fn snake_to_camel_case(ident: &str) -> String {
    ident.to_string().to_case(Case::Camel)
}

struct ServiceGenerator<'a> {
    service_ident: &'a Ident,
    methods: &'a [Method],
    server_ident: &'a Ident,
    methods_enum_ident: &'a Ident,
    method_idents: &'a [Ident],
}

impl<'a> ServiceGenerator<'a> {
    fn service_trait(&self) -> TokenStream2 {
        let ServiceGenerator {
            service_ident,
            server_ident,
            methods,
            ..
        } = self;

        let types_and_fns = methods.iter().map(|Method { ident, output, .. }| {
            quote! {
                fn #ident(self, ctx: String) #output;
            }
        });

        quote! {
            trait #service_ident: Sized {
                #( #types_and_fns )*

                fn serve(self) -> #server_ident<Self> {
                        #server_ident { service: self }
                }
            }

        }
    }

    fn server_struct(&self) -> TokenStream2 {
        let ServiceGenerator { server_ident, .. } = self;

        quote! {
            struct #server_ident<S> {
                service: S
            }
        }
    }

    fn impl_server_struct(&self) -> TokenStream2 {
        let ServiceGenerator {
            server_ident,
            service_ident,
            ..
        } = self;

        quote! {
            impl #server_ident<S>
                where S: #service_ident
            {
                fn serve(self) {
                    println!("Hello world");
                }
            }
        }
    }

    fn method_idents_enum(&self) -> TokenStream2 {
        let ServiceGenerator {
            methods_enum_ident,
            method_idents,
            ..
        } = self;
        quote! {
            enum #methods_enum_ident {
                #( #method_idents ),*
            }
        }
    }
}

impl<'a> ToTokens for ServiceGenerator<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend(vec![
            self.service_trait(),
            self.server_struct(),
            self.impl_server_struct(),
            self.method_idents_enum(),
        ])
    }
}
