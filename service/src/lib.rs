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

    let method_idents = methods.iter().map(|rpc| &rpc.ident).collect::<Vec<_>>();
    let camel_case_method_idents: &Vec<_> = &method_idents
        .iter()
        .map(|ident| snake_to_camel_case(&ident.unraw().to_string()))
        .collect();

    ServiceGenerator {
        service_ident: &ident,
        methods,
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
            client_ident,
            ..
        } = self;

        let types_and_fns = methods.iter().map(|Method { ident, output, .. }| {
            quote! {
                fn #ident(self) #output;
            }
        });

        quote! {
            trait #service_ident: Sized {
                #( #types_and_fns )*

                fn serve(self) -> #server_ident<Self> {
                    #server_ident { service: self }
                }

                fn build_client(self) -> #client_ident {
                    #client_ident::build()
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
            ..
        } = self;

        quote! {
            impl<S> server::HandleIncoming for #server_ident<S>
                where S: #service_ident
            {
                fn handle_request(self, req: Vec<u8>) {
                    let res: #methods_enum_ident = serde_json::from_slice(&req).unwrap();
                    match res {
                        #(
                            #methods_enum_ident::#camel_case_method_idents => {
                                 #service_ident::#method_idents(
                                        self.service,
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
            ..
        } = self;
        quote! {
            #[derive(serde::Serialize, serde::Deserialize, Debug)]
            enum #methods_enum_ident {
                #( #camel_case_method_idents ),*
            }
        }
    }

    fn client_struct(&self) -> TokenStream2 {
        let ServiceGenerator {
            client_ident,
            methods_enum_ident,
            ..
        } = self;

        quote! {
            #[allow(unused)]
            #[derive(Clone, Debug)]
            struct #client_ident {
                tx: tokio::sync::mpsc::Sender<Vec<u8>>
            }
        }
    }

    fn impl_client_struct(&self) -> TokenStream2 {
        let ServiceGenerator { client_ident, .. } = self;

        quote! {
            impl #client_ident {
                fn build() -> Self {
                    let tx = client::WsClient::connect("ws://127.0.0.1:3000");

                    Self { tx }
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
            ..
        } = self;

        quote! {
            impl #client_ident {
                #(
                    fn #method_idents(&self)
                      -> impl std::future::Future<Output = std::io::Result<()>> + '_
                    {
                        let req = serde_json::to_vec(
                            &#methods_enum_ident::#camel_case_method_idents
                        ).unwrap();

                        let fut = self.tx.send(req);

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
