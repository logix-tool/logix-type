use crate::{Shared, TokenStream2};
use quote::quote;

pub(crate) fn do_any(
    Shared {
        prefix,
        type_name_str,
        type_name,
        cr,
        impl_gen,
    }: &Shared,
    variants: impl IntoIterator<Item = syn::Variant>,
) -> (TokenStream2, TokenStream2) {
    let mut variants_desc = Vec::new();
    let mut variant_parsers = Vec::new();
    let mut variant_names_str = Vec::new();

    for variant in variants {
        let variant_name = variant.ident;
        let shared = Shared {
            prefix: quote!(#prefix #type_name::),
            cr: cr.clone(),
            type_name_str: variant_name.to_string(),
            type_name: variant_name,
            impl_gen: impl_gen.clone(),
        };
        let (value_desc, parse) = crate::derive_struct::do_any(&shared, variant.fields, true);
        variants_desc.push(value_desc);
        variant_parsers.push(parse);
        variant_names_str.push(shared.type_name_str);
    }

    (
        quote!(
            #cr::LogixValueDescriptor::Enum {
                variants: vec![#(#cr::LogixTypeDescriptor {
                    name: #variant_names_str,
                    doc: "",
                    value: #variants_desc
                 },)*],
            }
        ),
        quote!(
            match p.next_token()? {
                #((type_name_span, Token::Ident(#variant_names_str)) => {
                    #variant_parsers
                })*
                (span, token) => {
                    Err(ParseError::UnexpectedToken {
                        span,
                        while_parsing: #type_name_str,
                        wanted: Wanted::Tokens(&[#(Token::Ident(#variant_names_str),)*]),
                        got_token: token.token_type_name(),
                    })
                }
            }
        ),
    )
}
