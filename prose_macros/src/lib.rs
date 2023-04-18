use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Curries a method with a &self parameter and an unlocked XMPPClient.
#[proc_macro_attribute]
pub fn with_xmpp_client(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let sig = &input_fn.sig;
    let fn_name = &sig.ident;
    let inputs = sig.inputs.iter().skip(1);
    let output = &sig.output;
    let body = &input_fn.block;

    let expanded = quote! {
        pub async fn #fn_name(&self, #(#inputs),*) #output {
            let opt = &(*self.xmpp.read().await);
            let xmpp = opt.as_ref().ok_or(ClientError::NotConnected)?;
            #body
        }
    };

    TokenStream::from(expanded)
}
