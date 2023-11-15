// prose-core-client/prose-proc-macros
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use proc_macro::TokenStream;

use convert_case::{Case, Casing};
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, Attribute, Data, DeriveInput, Fields};

#[proc_macro_attribute]
pub fn entity(_attrs: TokenStream, stream: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(stream as DeriveInput);

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
    let mut collection_name = name.to_string().to_case(Case::Snake).to_lowercase();
    collection_name = collection_name
        .strip_suffix("_record")
        .unwrap_or(&collection_name)
        .to_string();

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

#[proc_macro_derive(InjectDependencies, attributes(inject))]
pub fn inject_deps(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);

    let Data::Struct(struct_data) = &input.data else {
        panic!("This macro only supports structs.")
    };

    let Fields::Named(fields) = &struct_data.fields else {
        panic!("This macro only supports structs with named fields.")
    };

    let field_initialization = fields
        .named
        .iter()
        .filter_map(|field| {
            let Some(ref ident) = field.ident else {
                return None;
            };

            let is_injected = field
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("inject"));

            if is_injected {
                Some(quote! { #ident: deps.#ident.clone() })
            } else {
                Some(quote! { #ident: Default::default() })
            }
        })
        .collect::<Vec<_>>();

    let name = &input.ident;
    let expanded = quote! {
        impl From<&crate::app::deps::AppDependencies> for #name {
            fn from(deps: &crate::app::deps::AppDependencies) -> Self {
                Self {
                    #(#field_initialization,)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(DependenciesStruct)]
pub fn dependencies_struct(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);

    let Data::Struct(struct_data) = &input.data else {
        panic!("This macro only supports structs.")
    };

    let Fields::Named(fields) = &struct_data.fields else {
        panic!("This macro only supports structs with named fields.")
    };

    let name = &input.ident;
    let dependencies_struct_name = format_ident!("{}Dependencies", name);

    let struct_fields = fields
        .named
        .iter()
        .filter_map(|field| {
            let Some(ref ident) = field.ident else {
                return None;
            };
            let field_type = &field.ty;
            Some(quote! { pub #ident: #field_type })
        })
        .collect::<Vec<_>>();

    let field_initialization = fields
        .named
        .iter()
        .filter_map(|field| {
            let Some(ref ident) = field.ident else {
                return None;
            };
            Some(quote! { #ident: deps.#ident })
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
        pub struct #dependencies_struct_name {
            #(#struct_fields,)*
        }

        impl From<#dependencies_struct_name> for #name {
            fn from(deps: #dependencies_struct_name) -> Self {
                Self {
                    #(#field_initialization,)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
