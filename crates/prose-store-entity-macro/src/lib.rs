use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Attribute, Data, DeriveInput, Fields};

#[proc_macro_attribute]
pub fn entity(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as DeriveInput);

    let Data::Struct(struct_data) = &input.data else {
        panic!("This macro only supports structs.")
    };

    let Fields::Named(fields) = &struct_data.fields else {
        panic!("This macro only supports structs with named fields.")
    };

    let id_type = fields
        .named
        .iter()
        .find(|field| field.ident.as_ref().map(|ident| ident.to_string()) == Some("id".to_string()))
        .map(|field| &field.ty);

    let Some(id_type) = id_type else {
        panic!("No field named 'id' found in struct.")
    };

    let name = &input.ident;
    let collection_name = name.to_string().to_lowercase();

    let derive_attr: Attribute =
        parse_quote! { #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)] };
    input.attrs.push(derive_attr);

    let expanded = quote! {
        #input

        impl prose_store::prelude::Entity for #name {
            type ID = #id_type;

            fn id(&self) -> &Self::ID {
                &self.id
            }

            fn collection() -> &'static str {
                #collection_name
            }
        }
    };

    TokenStream::from(expanded)
}
