// prose-core-client/prose-proc-macros
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use proc_macro::TokenStream;

use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, ItemFn};

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

            let type_str = quote! {#field_type};
            // Skip all fields that have types which do not start with 'Dyn' (i.e. not DynAppContext, etc.)
            if !format!("{type_str}").starts_with("Dyn") {
                return None;
            }

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

            let field_type = &field.ty;
            let type_str = quote! {#field_type};

            // Skip all fields that have types which do not start with 'Dyn' (i.e. not DynAppContext, etc.)
            if format!("{type_str}").starts_with("Dyn") {
                Some(quote! { #ident: deps.#ident })
            } else {
                Some(quote! { #ident: Default::default() })
            }
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
        #[derive(Clone)]
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

/// Multi-threaded test
#[proc_macro_attribute]
pub fn mt_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let expanded = quote! {
        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        #input
    };

    TokenStream::from(expanded)
}
