#![deny(warnings, clippy::all)]
#![allow(non_snake_case)] // NOTE(2024.03.29): There appear to be a bug triggering this even when set on the Types struct
mod derive_enum;
mod derive_struct;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[derive(Clone)]
struct Types {
    LogixTypeDescriptor: TokenStream2,
    LogixValueDescriptor: TokenStream2,
    LogixType: TokenStream2,
    LogixVfs: TokenStream2,
    LogixParser: TokenStream2,
    ParseResult: TokenStream2,
    ParseError: TokenStream2,
    Wanted: TokenStream2,
    Value: TokenStream2,
    Token: TokenStream2,
    Brace: TokenStream2,
    Delim: TokenStream2,
}

struct Shared<'a> {
    prefix: TokenStream2,
    type_name_str: String,
    type_name: syn::Ident,
    types: Types,
    impl_gen: syn::ImplGenerics<'a>,
}

/// Derives the LogixType trait
#[proc_macro_derive(LogixType)]
pub fn impl_logix_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let (impl_gen, ty_gen, where_gen) = input.generics.split_for_impl();

    let shared = Shared {
        prefix: quote!(),
        type_name_str: input.ident.to_string(),
        type_name: input.ident,
        types: Types {
            LogixTypeDescriptor: quote!(::logix_type::type_trait::LogixTypeDescriptor),
            LogixValueDescriptor: quote!(::logix_type::type_trait::LogixValueDescriptor),
            LogixType: quote!(::logix_type::LogixType),
            LogixVfs: quote!(::logix_vfs::LogixVfs),
            LogixParser: quote!(::logix_type::LogixParser),
            ParseResult: quote!(::logix_type::error::Result),
            ParseError: quote!(::logix_type::error::ParseError),
            Wanted: quote!(::logix_type::error::Wanted),
            Value: quote!(::logix_type::type_trait::Value),
            Token: quote!(::logix_type::token::Token),
            Brace: quote!(::logix_type::token::Brace),
            Delim: quote!(::logix_type::token::Delim),
        },
        impl_gen,
    };
    let Shared {
        prefix: _,
        type_name_str,
        type_name,
        types:
            Types {
                LogixTypeDescriptor,
                LogixType,
                LogixVfs,
                LogixParser,
                ParseResult,
                Value,
                ..
            },
        impl_gen,
    } = &shared;

    let (value_desc, parse) = match input.data {
        syn::Data::Struct(data) => derive_struct::do_any(&shared, data.fields, false),
        syn::Data::Enum(data) => derive_enum::do_any(&shared, data.variants),
        syn::Data::Union(..) => return quote!(compile_error!("Union is not supported")).into(),
    };

    let descriptor = quote!(
        #LogixTypeDescriptor {
            name: #type_name_str,
            doc: "",
            value: #value_desc,
        }
    );

    let tokens = quote! {
        impl #impl_gen #LogixType for #type_name #ty_gen #where_gen {
            fn descriptor() -> &'static #LogixTypeDescriptor{
                // NOTE(2023.11): Currently generics can't be used to make statics, so I need this work-around
                static RET: std::sync::OnceLock<#LogixTypeDescriptor> = std::sync::OnceLock::new();
                RET.get_or_init(|| #descriptor)
            }

            fn default_value() -> Option<Self> {
                None
            }

            fn logix_parse<FS: #LogixVfs>(p: &mut #LogixParser<FS>) -> #ParseResult<#Value<Self>> {
                #parse
            }
        }
    };

    tokens.into()
}
