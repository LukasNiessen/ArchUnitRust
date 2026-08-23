extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro]
pub fn fixture(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
