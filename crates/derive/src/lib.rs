//! Restate Rust SDK Macros

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use restate_sdk_types::endpoint_manifest::{
    Endpoint, Handler, HandlerName, HandlerType, ProtocolMode, Service, ServiceName, ServiceType,
};
use std::{fs::File, io::Write};
use syn::{Attribute, Expr, ImplItem, Item, ItemFn, ItemImpl, Lit, Type};

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

    let mut file = File::create("manifest.json").expect("Unable to create manifest file");
    file.write_all(manifest_json.as_bytes())
        .expect("Unable to write manifest file");

    let mut all = vec![];
    for service in &manifest.services {
        let mut m = handler_methods(service);
        all.append(&mut m);
    }

    let methods = manifest
        .services
        .iter()
        .flat_map(|service| handler_methods(service))
        .collect::<Vec<_>>();

    quote!(
        #endpoint
        async fn service(req: Request<Incoming>) -> restate::Result<Response<BoxBody<Bytes, Error>>> {
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
                let (http2conn, boxed_body) = Http2Connection::new(req);
                tokio::spawn(
                    async move { http2_handler::handle(crate::bundle::#service::#handler, http2conn).await },
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

    for item in service.items.iter() {
        match item {
            ImplItem::Const(_) => {}
            ImplItem::Fn(handler) => {
                if handler.sig.asyncness.is_some() {
                    println!("Handler {}", handler.sig.ident.to_string());
                }
            }
            _ => {
                panic!("Only consts and fns");
            }
        }
    }
    quote!(
        pub struct #service_name;
        #service
        impl ServiceHandler for #service_name {
            fn name(&self) -> &'static str {
                Self::NAME
            }

            fn handlers(&self) -> &'static [&'static str] {
                &["service", "greet"]
            }
        }
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
