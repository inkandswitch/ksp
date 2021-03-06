#![forbid(unsafe_code, future_incompatible, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![recursion_limit = "512"]

// Had to add disable unused_imports warnings due to rust-analyzer bug
// See: https://github.com/rust-analyzer/rust-analyzer/issues/3860
#[allow(unused_imports)]
use proc_macro::TokenStream;
#[allow(unused_imports)]
use quote::{quote, quote_spanned};
#[allow(unused_imports)]
use syn::spanned::Spanned;

#[cfg(not(test))] // NOTE: exporting main breaks tests, we should file an issue.
#[proc_macro_attribute]
pub fn wait(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    let ret = &input.sig.output;
    let inputs = &input.sig.inputs;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;

    if input.sig.asyncness.is_none() {
        return TokenStream::from(quote_spanned! { input.span() =>
            compile_error!("the async keyword is missing from the function declaration"),
        });
    }

    let result = quote! {
        #(#attrs)*
        fn #name(#inputs) #ret {
          async_std::task::block_on(async {
            #body
          })
        }
    };

    result.into()
}
