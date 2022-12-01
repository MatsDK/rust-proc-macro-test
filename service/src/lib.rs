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
    FnArg, Ident, Pat, PatType, ReturnType, Token, Visibility,
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
        let _vis: Visibility = input.parse()?;
        <Token![trait]>::parse(input)?;
        let ident: Ident = input.parse()?;

        let content;
        braced!(content in input);

        let mut methods = Vec::new();
        while !content.is_empty() {
            methods.push(<Method>::parse(&content)?);
        }

        for method in &methods {
            if method.ident == "serve" {
                Err(syn::Error::new(
                    method.ident.span(),
                    format!("method name conflicts with generated fn `{ident}::serve`"),
                ))?;
            }
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

    let args: &[&[PatType]] = &methods.iter().map(|rpc| &*rpc.args).collect::<Vec<_>>();
    let method_idents = methods.iter().map(|rpc| &rpc.ident).collect::<Vec<_>>();
    let camel_case_method_idents: &Vec<_> = &method_idents
        .iter()
        .map(|ident| snake_to_camel_case(&ident.unraw().to_string()))
        .collect();

    ServiceGenerator {
        service_ident: &ident,
        methods,
        args,
        server_ident: &format_ident!("{}Server", ident),
        methods_enum_ident: &format_ident!("{}Methods", ident),
        method_idents: &method_idents,
        camel_case_method_idents: &methods
            .iter()
            .zip(camel_case_method_idents.iter())
            .map(|(method, name)| Ident::new(&name.to_string(), method.ident.span()))
            .collect::<Vec<_>>(),
        client_ident: &format_ident!("{}Client", ident),
    }
    .into_token_stream()
    .into()
}

fn snake_to_camel_case(ident: &str) -> String {
    ident.to_string().to_case(Case::UpperCamel)
}

struct ServiceGenerator<'a> {
    service_ident: &'a Ident,
    methods: &'a [Method],
    args: &'a [&'a [PatType]],
    server_ident: &'a Ident,
    client_ident: &'a Ident,
    methods_enum_ident: &'a Ident,
    camel_case_method_idents: &'a [Ident],
    method_idents: &'a [&'a Ident],
}

impl<'a> ServiceGenerator<'a> {
    fn service_trait(&self) -> TokenStream2 {
        let ServiceGenerator {
            service_ident,
            server_ident,
            methods,
            ..
        } = self;

        let types_and_fns = methods.iter().map(
            |Method {
                 ident,
                 output,
                 args,
             }| {
                quote! {
                    fn #ident(self, #( #args ),*) #output;
                }
            },
        );

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
            #[derive(Clone)]
            struct #server_ident<S> {
                service: S
            }
        }
    }

    fn impl_server_struct(&self) -> TokenStream2 {
        let ServiceGenerator {
            server_ident,
            service_ident,
            method_idents,
            methods_enum_ident,
            camel_case_method_idents,
            args,
            ..
        } = self;

        let arg_pats: &[Vec<&Pat>] = &args
            .iter()
            .map(|args| args.iter().map(|arg| &*arg.pat).collect())
            .collect::<Vec<_>>();

        quote! {
            impl<S> client::HandleIncoming for #server_ident<S>
                where S: #service_ident
            {
                fn handle_incoming_event(self, req: Vec<u8>) {
                    let res: #methods_enum_ident = serde_json::from_slice(&req).unwrap();
                    match res {
                        #(
                            #methods_enum_ident::#camel_case_method_idents{ #( #arg_pats ),* } => {
                                 #service_ident::#method_idents(
                                        self.service, #( #arg_pats ),*
                                );
                            }
                        )*
                    }
                }
            }
        }
    }

    fn method_idents_enum(&self) -> TokenStream2 {
        let ServiceGenerator {
            methods_enum_ident,
            camel_case_method_idents,
            args,
            ..
        } = self;
        quote! {
            #[derive(serde::Serialize, serde::Deserialize, Debug)]
            enum #methods_enum_ident {
                #( #camel_case_method_idents{ #( #args ),* } ),*
            }
        }
    }

    fn client_struct(&self) -> TokenStream2 {
        let ServiceGenerator { client_ident, .. } = self;

        quote! {
            #[allow(unused)]
            struct #client_ident(client::Channel);
        }
    }

    fn impl_client_struct(&self) -> TokenStream2 {
        let ServiceGenerator { client_ident, .. } = self;

        quote! {
            impl #client_ident {
                async fn new<A, R>(addr: A, resolvers: R) -> std::io::Result<Self>
                where
                    A: std::borrow::Borrow<str> + Send + 'static,
                    R: client::HandleIncoming + Clone + Send + 'static
                {
                    let channel = client::WsClient::connect(addr, resolvers).await?;

                    Ok(Self(channel))
                }
            }
        }
    }

    fn impl_methods_for_client(&self) -> TokenStream2 {
        let ServiceGenerator {
            client_ident,
            methods_enum_ident,
            method_idents,
            camel_case_method_idents,
            args,
            ..
        } = self;

        let arg_pats: &[Vec<&Pat>] = &args
            .iter()
            .map(|args| args.iter().map(|arg| &*arg.pat).collect())
            .collect::<Vec<_>>();

        quote! {
            impl #client_ident {
                #(
                    fn #method_idents(&self, #( #args ),*)
                      -> impl std::future::Future<Output = std::io::Result<()>> + '_
                    {
                        let req = serde_json::to_vec(
                            &#methods_enum_ident::#camel_case_method_idents{ #( #arg_pats ),* }
                        ).unwrap();
                        let fut = self.0.tx.send(req);

                        async move {
                            match fut.await {
                                Ok(_) => std::io::Result::Ok(()),
                                _ =>  {
                                    unreachable!()
                                }
                            }
                        }
                    }
                )*
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
            self.client_struct(),
            self.impl_client_struct(),
            self.impl_methods_for_client(),
        ])
    }
}
