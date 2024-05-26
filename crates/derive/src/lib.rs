//! Restate Rust SDK Macros

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use restate_sdk_types::endpoint_manifest::{ServiceName, ServiceType};
use std::{fs::File, io::Write, str::FromStr};
use syn::{ImplItem, Item, ItemFn, ItemImpl, Type};

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
    let endpoint_name = match &endpoint {
        Item::Mod(module) => {
            if let Some((_, items)) = &module.content {
                for item in items {
                    match item {
                        Item::Impl(service) => {
                            if service.attrs.iter().any(|attribute| {
                                attribute
                                    .meta
                                    .path()
                                    .segments
                                    .iter()
                                    .map(|s| s.ident.to_string())
                                    .collect::<Vec<_>>()
                                    .join("::")
                                    .eq("restate::service")
                            }) {
                                println!("Found service {:?}", service)
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
    let file = std::fs::read_dir(".restate").unwrap();
    file.for_each(|f| println!("File: {:?}", f.unwrap()));
    quote!(
        #endpoint
    )
    .into()
}

#[proc_macro_attribute]
#[cfg(not(test))]
pub fn service(args: TokenStream, item: TokenStream) -> TokenStream {
    let service = syn::parse_macro_input!(item as ItemImpl);
    let service_name = match service.self_ty.as_ref() {
        Type::Path(path) => {
            let service = path.path.segments[0].ident.to_string();
            let service_name = format_ident!("{}", service);
            println!("{}", service_name);
            format_ident!("{}", service_name.to_string())
        }
        _ => {
            panic!("Only on impl struct");
        }
    };

    let service_name_str = service_name.to_string();
    let service_json = restate_sdk_types::endpoint_manifest::Service {
        handlers: vec![],
        name: ServiceName::from_str(&service_name_str).unwrap(),
        ty: ServiceType::Service,
    };

    let service_json = serde_json::to_string_pretty(&service_json).unwrap();

    let mut file =
        File::create(format!(".restate/{}.json", service_name_str)).expect("Unable to create file");
    file.write_all(service_json.as_bytes())
        .expect("Unable to write data");

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
