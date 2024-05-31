//! Restate Rust SDK Macros

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use restate_sdk_types::endpoint_manifest::{
    Endpoint, Handler, HandlerName, HandlerType, ProtocolMode, Service, ServiceName, ServiceType,
};
use syn::{
    parse_quote, token::Brace, Attribute, Block, Expr, FnArg, ImplItem, ImplItemFn, Item, ItemFn, ItemImpl,
    Lit, Pat, Receiver, Stmt, Type,
};

#[proc_macro_attribute]
#[cfg(not(test))]
pub fn main(args: TokenStream, item: TokenStream) -> TokenStream {
    let main = syn::parse_macro_input!(item as ItemFn);
    println!("Main name: {:?}", main.sig.ident.to_string());
    let body = main.block;
    quote!(
        #[tokio::main]
        async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            #body
        }
    )
    .into()
}

#[proc_macro_attribute]
#[cfg(not(test))]
pub fn bundle(args: TokenStream, item: TokenStream) -> TokenStream {
    let endpoint = syn::parse_macro_input!(item as Item);
    let mut services = vec![];
    match &endpoint {
        Item::Mod(module) => {
            println!("Module path {:?}", module.ident.to_string());
            if let Some((_, items)) = &module.content {
                for item in items {
                    match item {
                        Item::Impl(item) => {
                            if match_attribute("restate::service", &item.attrs) {
                                services.push(create_service(&item))
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        _ => {
            panic!("Only on mod")
        }
    };

    // TODO: Read from attributes
    let manifest = Endpoint {
        max_protocol_version: 1,
        min_protocol_version: 1,
        protocol_mode: Some(ProtocolMode::BidiStream),
        services,
    };

    let manifest_json = serde_json::to_string(&manifest).unwrap();

    /*
    use std::{fs::File, io::Write};
    let mut file = File::create("manifest.json").expect("Unable to create manifest file");
    file.write_all(manifest_json.as_bytes())
        .expect("Unable to write manifest file");
     */

    let methods = manifest
        .services
        .iter()
        .flat_map(|service| handler_methods(service))
        .collect::<Vec<_>>();

    quote!(
        #endpoint
        use restate::{
            empty, full, http2_handler, setup_connection, BodyExt, BoxBody, Bytes, Incoming, Method, Request,
            Response, StatusCode,
        };
        pub async fn service(req: Request<Incoming>) -> restate::Result<Response<BoxBody<Bytes, anyhow::Error>>> {
            match (req.method(), req.uri().path()) {
                (&Method::POST, "/discover") => {
                    let manifest = #manifest_json;
                    let response = Response::builder()
                        .status(StatusCode::OK)
                        .header("content-type", "application/json")
                        .header("x-restate-server", "restate-sdk-rust/0.1.0")
                        .body(full(manifest).map_err(|e| e.into()).boxed())
                        .unwrap();
                    Ok(response)
                }
                #(#methods)*
                // Return the 404 Not Found for other routes.
                _ => {
                    println!("{}, {}", req.method(), req.uri().path());
                    let response = Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(empty().map_err(|e| e.into()).boxed())
                        .unwrap();
                    Ok(response)
                }
            }
        }
    )
    .into()
}

fn handler_methods(service: &Service) -> Vec<proc_macro2::TokenStream> {
    let mut routes = vec![];
    for handler in &service.handlers {
        let service = service.name.to_string();
        let handler = handler.name.to_string();
        let route = format!("/invoke/{}/{}", service, handler);
        let service = format_ident!("{}", service);
        let handler = format_ident!("{}", handler);
        routes.push(quote!(
           (&Method::POST, #route) => {
                let (receiver, sender, boxed_body) = setup_connection(req);
                tokio::spawn(
                    async move {
                        http2_handler::handle(bundle::#service::#handler, None, receiver, sender, false).await;
                        println!("Invocation task completed");
                    },
                );
                let response = Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/restate")
                    .header("x-restate-server", "restate-sdk-rust/0.1.0")
                    .body(boxed_body)
                    .unwrap();
                Ok(response)
            }
        ));
    }
    routes
}

#[proc_macro_attribute]
#[cfg(not(test))]
pub fn service(args: TokenStream, item: TokenStream) -> TokenStream {
    let service = syn::parse_macro_input!(item as ItemImpl);
    let service_name = match service.self_ty.as_ref() {
        Type::Path(path) => {
            let service = path.path.segments[0].ident.to_string();
            format_ident!("{}", service)
        }
        _ => {
            panic!("Only on impl struct");
        }
    };
    let mut methods = vec![];
    for item in service.items.iter() {
        match item {
            ImplItem::Const(_) => {}
            ImplItem::Fn(handler) => {
                if handler.sig.asyncness.is_some() {
                    if match_attribute("restate::handler", &handler.attrs) {
                        println!("Handler {}", handler.sig.ident.to_string());
                        let method = create_service_client_fn(service_name.clone(), handler);
                        methods.push(method);
                    }
                }
            }
            _ => {
                panic!("Only consts and fns");
            }
        }
    }
    let service_client = format_ident!("{}ClientImpl", service_name.to_string());
    let service_client_ext = format_ident!("{}ClientExt", service_name.to_string());
    let service_client_indent = service_name.to_string().to_case(Case::Snake);
    let service_client_indent = format_ident!("{}_client", service_client_indent);
    quote!(
        pub struct #service_name;
        #service
        struct #service_client<'a> {
            ctx: &'a Context,
        }
        impl<'a> #service_client<'a> {
           #(#methods)*
        }
        trait #service_client_ext {
            fn #service_client_indent(&self) -> #service_client;
        }

        impl #service_client_ext for Context {
            fn #service_client_indent(&self) -> #service_client {
                #service_client { ctx: &self }
            }
        }
    )
    .into()

    /*
    impl ServiceHandler for #service_name {
            fn name(&self) -> &'static str {
                Self::NAME
            }

            fn handlers(&self) -> &'static [&'static str] {
                &["service", "greet"]
            }
        }
     */
}

fn create_service_client_fn(service: proc_macro2::Ident, handler: &ImplItemFn) -> proc_macro2::TokenStream {
    let mut client_fn = handler.clone();
    client_fn.attrs.clear();
    let mut signature = &mut client_fn.sig;
    let first = signature.inputs.first_mut().unwrap();
    *first = FnArg::Receiver(Receiver {
        attrs: vec![],
        reference: None,
        mutability: None,
        self_token: Default::default(),
        colon_token: None,
        ty: Box::new(Type::Verbatim(quote!(Self))),
    });
    let method = &signature.ident;
    let last = signature.inputs.last().unwrap();
    let parameter = match last {
        FnArg::Receiver(_) => {
            panic!("There should be input");
        }
        FnArg::Typed(typed) => match *typed.pat {
            Pat::Ident(ref ident) => &ident.ident,
            _ => {
                panic!("There should be input");
            }
        },
    };

    let service_literal = service.to_string();
    let method_literal = method.to_string();
    let stmts: Vec<Stmt> = parse_quote!(
        self.ctx
            .invoke(
                #service::#method,
                #service_literal.to_string(),
                #method_literal.to_string(),
                #parameter,
                None,
            )
            .await
    );

    client_fn.block = Block {
        brace_token: Brace::default(),
        stmts,
    };

    quote! (
        #client_fn
    )
    .into()
}

fn create_service(item: &ItemImpl) -> Service {
    let service_name = match item.self_ty.as_ref() {
        Type::Path(path) => path.path.segments[0].ident.to_string(),
        _ => {
            panic!("Only on impl struct");
        }
    };
    let mut service = Service {
        handlers: vec![],
        name: ServiceName::try_from(&service_name).unwrap(),
        ty: ServiceType::Service,
    };
    for item in item.items.iter() {
        match item {
            ImplItem::Const(property) => {
                let name = property.ident.to_string();
                if name.eq("TYPE") {
                    // Should be only static string
                    let value = match &property.expr {
                        Expr::Lit(expr) => match &expr.lit {
                            Lit::Str(value) => value.value(),
                            _ => {
                                panic!("Only literal string");
                            }
                        },
                        _ => {
                            panic!("Only literal string")
                        }
                    };
                    service.ty = ServiceType::try_from(value).unwrap();
                    println!("Found property {:?}", service);
                }
            }
            ImplItem::Fn(handler) => {
                if handler.sig.asyncness.is_some() {
                    if match_attribute("restate::handler", &handler.attrs) {
                        let name = handler.sig.ident.to_string();
                        service.handlers.push(Handler {
                            input: None,
                            name: HandlerName::try_from(name).unwrap(),
                            output: None,
                            ty: Some(HandlerType::Exclusive),
                        })
                    }
                }
            }
            _ => {
                panic!("Only consts and fns");
            }
        }
    }
    service
}

fn match_attribute(name: &'static str, attrs: &Vec<Attribute>) -> bool {
    attrs.iter().any(|attribute| {
        attribute
            .meta
            .path()
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::")
            .eq(name)
    })
}

#[proc_macro_attribute]
#[cfg(not(test))]
pub fn object(args: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
#[cfg(not(test))]
pub fn workflow(args: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
#[cfg(not(test))]
pub fn handler(args: TokenStream, item: TokenStream) -> TokenStream {
    let handler = syn::parse_macro_input!(item as ItemFn);
    quote!(
        #handler
    )
    .into()
}
