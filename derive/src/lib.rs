use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(LogixType)]
pub fn impl_logix_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let cr = quote!(logix_mold::type_trait);
    let type_name = input.ident;
    let type_name_str = type_name.to_string();

    let (value_desc, parse) = match input.data {
        syn::Data::Struct(data) => {
            let mut members_desc = Vec::new();
            let mut member_names = Vec::new();
            let mut member_str_names = Vec::new();
            let mut member_tmp_types = Vec::new();
            let mut member_tmp_init = Vec::new();
            let mut member_tmp_parse = Vec::new();
            let mut member_tmp_assign = Vec::new();
            match data.fields {
                syn::Fields::Named(fields) => {
                    for field in fields.named {
                        let fname = field.ident.unwrap();
                        let fname_str = fname.to_string();
                        let ty = field.ty;
                        let ftype = quote!(<#ty as #cr::LogixType>);
                        members_desc.push(quote!((#fname_str, #ftype::DESCRIPTOR)));
                        member_tmp_types.push(ty);
                        member_tmp_init.push(quote!(None));
                        member_tmp_parse.push(quote!(tmp.#fname = Some(#ftype::logix_parse(p)?)));
                        member_tmp_assign.push(quote!(tmp.#fname.ok_or_else(|| ParseError::missing_member(#ftype::DESCRIPTOR, #fname_str))?));
                        member_names.push(fname);
                        member_str_names.push(fname_str);
                    }
                }
                syn::Fields::Unnamed(..) => todo!("Implement Struct(..)"),
                syn::Fields::Unit => {}
            }
            (
                quote!(
                    #cr::LogixValueDescriptor::Struct {
                        members: &[#(#members_desc,)*],
                    }
                ),
                quote!(
                    match p.next_token()? {
                        Some((_, Token::Ident(#type_name_str))) => {
                            match p.next_token()? {
                                Some((_, Token::BraceStart(Brace::Curly))) => {
                                    struct Tmp {
                                        #(#member_names: Option<#member_tmp_types>,)*
                                    }
                                    let mut tmp = Tmp {
                                        #(#member_names: #member_tmp_init,)*
                                    };
                                    'parse_members: loop {
                                        match p.next_token()? {
                                            #(Some((_, Token::Ident(#member_str_names))) => match p.next_token()? {
                                                Some((_, Token::Colon)) => {
                                                    #member_tmp_parse;
                                                    match p.next_token()? {
                                                        Some((_, Token::Newline)) => {}
                                                        unk => return Err(ParseError::unexpected_token(Self::DESCRIPTOR, "end of line", unk)),
                                                    }
                                                }
                                                unk => return Err(ParseError::unexpected_token(Self::DESCRIPTOR, ":", unk)),
                                            })*
                                            Some((_, Token::BraceEnd(Brace::Curly))) => break 'parse_members,
                                            unk => return Err(ParseError::unexpected_token(Self::DESCRIPTOR, concat!(#(#member_str_names, ",",)*), unk)),
                                        }
                                    }
                                    Ok(Self {
                                        #(#member_names: #member_tmp_assign,)*
                                    })
                                }
                                unk => Err(ParseError::unexpected_token(Self::DESCRIPTOR, "{", unk)),
                            }
                        }
                        unk => Err(ParseError::unexpected_token(Self::DESCRIPTOR, #type_name_str, unk)),
                    }
                ),
            )
        }
        syn::Data::Enum(..) => todo!("Implement support for enums"),
        syn::Data::Union(..) => todo!("Implement unions"),
    };

    quote! {
        impl #cr::LogixType for #type_name {
            const DESCRIPTOR: &'static #cr::LogixTypeDescriptor = &#cr::LogixTypeDescriptor {
                name: #type_name_str,
                doc: "",
                value: #value_desc,
            };

            fn logix_parse<R: std::io::Read>(p: &mut #cr::LogixParser<R>) -> #cr::Result<Self> {
                use #cr::{Token, ParseError, Brace, LogixType};
                #parse
            }
        }
    }
    .into()
}
