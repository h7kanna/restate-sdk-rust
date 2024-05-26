//! Restate Rust SDK Macros

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::fs::File;
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
    let file = File::create(format!(".restate/{}.txt", service_name.to_string()));
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
pub fn bundle(args: TokenStream, item: TokenStream) -> TokenStream {
    let endpoint = syn::parse_macro_input!(item as Item);
    let endpoint_name = match &endpoint {
        Item::Mod(module) => module.ident.clone(),
        _ => {
            panic!("Only on mod")
        }
    };
    println!("Module name: {:?}", endpoint_name.to_string());
    let file = std::fs::read_dir(".restate").unwrap();
    file.for_each(|f| println!("File: {:?}", f.unwrap()));
    quote!(
        #endpoint
    )
    .into()
}
