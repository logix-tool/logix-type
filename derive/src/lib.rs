mod derive_enum;
mod derive_struct;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

struct Shared {
    prefix: TokenStream2,
    type_name_str: String,
    type_name: syn::Ident,
    cr: TokenStream2,
}

#[proc_macro_derive(LogixType)]
pub fn impl_logix_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let shared = Shared {
        prefix: quote!(),
        type_name_str: input.ident.to_string(),
        type_name: input.ident,
        cr: quote!(logix_type::__private),
    };
    let Shared {
        prefix: _,
        type_name_str,
        type_name,
        cr,
    } = &shared;

    let (value_desc, parse) = match input.data {
        syn::Data::Struct(data) => derive_struct::do_any(&shared, data.fields, false),
        syn::Data::Enum(data) => derive_enum::do_any(&shared, data.variants),
        syn::Data::Union(..) => return quote!(compile_error!("Union is not supported")).into(),
    };

    let tokens = quote! {
        impl #cr::LogixType for #type_name {
            const DESCRIPTOR: &'static #cr::LogixTypeDescriptor = &#cr::LogixTypeDescriptor {
                name: #type_name_str,
                doc: "",
                value: #value_desc,
            };

            fn logix_parse<FS: #cr::LogixVfs>(p: &mut #cr::LogixParser<FS>) -> #cr::Result<#cr::Value<Self>> {
                use #cr::{Token, ParseError, Brace, LogixType, Wanted, Delim};
                #parse
            }
        }
    };

    tokens.into()
}
