#![deny(warnings, clippy::all)]

mod derive_enum;
mod derive_struct;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

struct Shared<'a> {
    prefix: TokenStream2,
    type_name_str: String,
    type_name: syn::Ident,
    cr: TokenStream2,
    impl_gen: syn::ImplGenerics<'a>,
}

#[proc_macro_derive(LogixType)]
pub fn impl_logix_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let (impl_gen, ty_gen, where_gen) = input.generics.split_for_impl();

    let shared = Shared {
        prefix: quote!(),
        type_name_str: input.ident.to_string(),
        type_name: input.ident,
        cr: quote!(logix_type::__private),
        impl_gen,
    };
    let Shared {
        prefix: _,
        type_name_str,
        type_name,
        cr,
        impl_gen,
    } = &shared;

    let (value_desc, parse) = match input.data {
        syn::Data::Struct(data) => derive_struct::do_any(&shared, data.fields, false),
        syn::Data::Enum(data) => derive_enum::do_any(&shared, data.variants),
        syn::Data::Union(..) => return quote!(compile_error!("Union is not supported")).into(),
    };

    let descriptor = quote!(
        #cr::LogixTypeDescriptor {
            name: #type_name_str,
            doc: "",
            value: #value_desc,
        }
    );

    let tokens = quote! {
        impl #impl_gen #cr::LogixType for #type_name #ty_gen #where_gen {
            fn descriptor() -> &'static #cr::LogixTypeDescriptor{
                // NOTE(2023.11): Currently generics can't be used to make statics, so I need this work-around
                static RET: std::sync::OnceLock<#cr::LogixTypeDescriptor> = std::sync::OnceLock::new();
                RET.get_or_init(|| #descriptor)
            }

            fn logix_parse<FS: #cr::LogixVfs>(p: &mut #cr::LogixParser<FS>) -> #cr::Result<#cr::Value<Self>> {
                use #cr::{Token, ParseError, Brace, LogixType, Wanted, Delim};
                #parse
            }
        }
    };

    tokens.into()
}
