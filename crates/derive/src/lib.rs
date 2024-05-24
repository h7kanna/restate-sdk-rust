//! Restate Rust SDK Macros

use proc_macro::TokenStream;
use quote::quote;
use syn::ItemFn;

#[proc_macro_attribute]
#[cfg(not(test))]
pub fn main(args: TokenStream, item: TokenStream) -> TokenStream {
    let main = syn::parse_macro_input!(item as ItemFn);
    let body = main.block;
    quote!(
        #[tokio::main]
        async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            #body
        }
    )
    .into()
}
