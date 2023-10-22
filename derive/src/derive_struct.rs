use crate::{Shared, TokenStream2};
use quote::quote;

pub(crate) fn do_any(shared: &Shared, fields: syn::Fields) -> (TokenStream2, TokenStream2) {
    match fields {
        syn::Fields::Named(fields) => do_named(shared, fields),
        syn::Fields::Unnamed(fields) => do_unnamed(shared, fields),
        syn::Fields::Unit => do_unit(shared),
    }
}

pub(crate) fn do_unit(
    Shared {
        prefix,
        type_name_str,
        type_name,
        cr,
    }: &Shared,
) -> (TokenStream2, TokenStream2) {
    (
        quote!(
            #cr::LogixValueDescriptor::Struct {
                members: &[],
            }
        ),
        quote!(
            let type_name_span = p.req_token(#type_name_str, Token::Ident(#type_name_str))?;
            Ok(#cr::Value {
                value: #prefix #type_name,
                span: type_name_span,
            })
        ),
    )
}

pub(crate) fn do_named(
    Shared {
        prefix,
        type_name_str,
        type_name,
        cr,
    }: &Shared,
    fields: syn::FieldsNamed,
) -> (TokenStream2, TokenStream2) {
    let mut members_desc = Vec::new();
    let mut member_names = Vec::new();
    let mut member_str_names = Vec::new();
    let mut member_tmp_types = Vec::new();
    let mut member_tmp_init = Vec::new();
    let mut member_tmp_parse = Vec::new();
    let mut member_tmp_assign = Vec::new();

    for field in fields.named {
        let fname = field.ident.unwrap();
        let fname_str = fname.to_string();
        let ty = field.ty;
        members_desc.push(quote!((#fname_str, <#ty as #cr::LogixType>::DESCRIPTOR)));
        member_tmp_init.push(quote!(None));
        member_tmp_parse.push(quote!(
            if tmp.#fname.is_some() {
                return Err(ParseError::duplicate_member(#type_name_str, #fname_str));
            }
            tmp.#fname = Some(<#ty as #cr::LogixType>::logix_parse(p)?.value)
        ));
        member_tmp_types.push(ty);
        member_tmp_assign.push(
            quote!(tmp.#fname.ok_or_else(|| ParseError::missing_member(#type_name_str, #fname_str))?),
        );
        member_names.push(fname);
        member_str_names.push(fname_str);
    }

    (
        quote!(
            #cr::LogixValueDescriptor::Struct {
                members: &[#(#members_desc,)*],
            }
        ),
        quote!(
            match p.next_token()? {
                Some((type_name_span, Token::Ident(#type_name_str))) => {
                    struct Tmp {
                        #(#member_names: Option<#member_tmp_types>,)*
                    }
                    let mut tmp = Tmp {
                        #(#member_names: #member_tmp_init,)*
                    };

                    p.req_token(#type_name_str, Token::BraceStart(Brace::Curly))?;
                    p.req_token(#type_name_str, Token::Newline)?;

                    'parse_members: loop {
                        match p.next_token()? {
                            #(Some((_, Token::Ident(#member_str_names))) => {
                                p.req_token(#type_name_str, Token::Colon)?;
                                #member_tmp_parse;
                                p.req_token(#type_name_str, Token::Newline)?;
                            })*
                            Some((_, Token::BraceEnd(Brace::Curly))) => break 'parse_members,
                            Some((span, token)) => return Err(ParseError::UnexpectedToken {
                                span,
                                while_parsing: #type_name_str,
                                wanted: Wanted::Tokens(&[
                                    Token::BraceEnd(Brace::Curly),
                                    #(Token::Ident(#member_str_names),)*
                                ]),
                                got_token: token.token_type_name(),
                            }),
                            None => todo!("Unexpected end of file"),
                        }
                    }
                    Ok(#cr::Value {
                        value: #prefix #type_name {
                            #(#member_names: #member_tmp_assign,)*
                        },
                        span: type_name_span,
                    })
                }
                Some((span, token)) => Err(ParseError::UnexpectedToken {
                    span,
                    while_parsing: #type_name_str,
                    wanted: Wanted::Token(Token::Ident(#type_name_str)),
                    got_token: token.token_type_name(),
                }),
                None => todo!("Unexpected end of file"),
            }
        ),
    )
}

pub(crate) fn do_unnamed(
    Shared {
        prefix,
        type_name_str,
        type_name,
        cr,
    }: &Shared,
    fields: syn::FieldsUnnamed,
) -> (TokenStream2, TokenStream2) {
    let mut members_desc = Vec::new();
    let mut member_indices = Vec::new();
    let mut member_str_names = Vec::new();
    let mut member_parse = Vec::new();

    for (i, field) in fields.unnamed.into_iter().enumerate() {
        let ty = field.ty;
        let fname_str = format!("#{i}");
        members_desc.push(quote!(<#ty as #cr::LogixType>::DESCRIPTOR));
        member_parse.push(quote!(<#ty as #cr::LogixType>::logix_parse(p)?.value));
        member_indices.push(i);
        member_str_names.push(fname_str);
    }

    let last_member_parse = member_parse.pop().into_iter();

    (
        quote!(
            #cr::LogixValueDescriptor::Tuple {
                members: &[#(#members_desc,)*],
            }
        ),
        quote!(
            let type_name_span = p.req_token(#type_name_str, Token::Ident(#type_name_str))?;
            p.req_token(#type_name_str, Token::BraceStart(Brace::Paren))?;
            Ok(#cr::Value {
                value: #prefix #type_name (
                    #({
                        let value = #member_parse;

                        p.req_token(#type_name_str, Token::Comma)?;

                        value
                    },)*
                    #({
                        let value = #last_member_parse;

                        match p.next_token()? {
                            Some((_, Token::Comma)) => {
                                p.req_token(#type_name_str, Token::BraceEnd(Brace::Paren))?;
                            },
                            Some((_, Token::BraceEnd(Brace::Paren))) => {},
                            Some((span, token)) => return Err(ParseError::UnexpectedToken {
                                span,
                                while_parsing: #type_name_str,
                                wanted: Wanted::Tokens(&[Token::Comma, Token::BraceEnd(Brace::Paren)]),
                                got_token: token.token_type_name(),
                            }),
                            None => todo!("Unexpected end of file"),
                        }

                        value
                    },)*
                ),
                span: type_name_span,
            })
        ),
    )
}
