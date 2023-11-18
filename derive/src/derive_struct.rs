use crate::{Shared, TokenStream2};
use quote::quote;

pub(crate) fn do_any(
    shared: &Shared,
    fields: syn::Fields,
    skip_struct_ident: bool,
) -> (TokenStream2, TokenStream2) {
    match fields {
        syn::Fields::Named(fields) => do_named(shared, fields, skip_struct_ident),
        syn::Fields::Unnamed(fields) => do_unnamed(shared, fields, skip_struct_ident),
        syn::Fields::Unit => do_unit(shared, skip_struct_ident),
    }
}

pub(crate) fn do_unit(
    Shared {
        prefix,
        type_name_str,
        type_name,
        cr,
        impl_gen: _,
    }: &Shared,
    skip_struct_ident: bool,
) -> (TokenStream2, TokenStream2) {
    let req_struct_ident = if skip_struct_ident {
        quote!()
    } else {
        quote!(let type_name_span = p.req_token(#type_name_str, Token::Ident(#type_name_str))?;)
    };

    (
        quote!(
            #cr::LogixValueDescriptor::Struct {
                members: vec![],
            }
        ),
        quote!(
            #req_struct_ident
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
        impl_gen,
    }: &Shared,
    fields: syn::FieldsNamed,
    skip_struct_ident: bool,
) -> (TokenStream2, TokenStream2) {
    let mut members_desc = Vec::new();
    let mut member_names = Vec::new();
    let mut member_str_names = Vec::new();
    let mut member_tmp_types = Vec::new();
    let mut member_tmp_init = Vec::new();
    let mut member_tmp_parse = Vec::new();
    let mut member_tmp_assign = Vec::new();

    let req_struct_ident = if skip_struct_ident {
        quote!()
    } else {
        quote!(let type_name_span = p.req_token(#type_name_str, Token::Ident(#type_name_str))?;)
    };

    for field in fields.named {
        let fname = field.ident.unwrap();
        let fname_str = fname.to_string();
        let ty = field.ty;
        members_desc.push(quote!((#fname_str, <#ty as #cr::LogixType>::descriptor())));
        member_tmp_init.push(quote!(None));
        member_tmp_parse.push(quote!(
            if tmp.#fname.is_some() {
                return Err(ParseError::DuplicateStructMember {
                    span,
                    type_name: #type_name_str,
                    member: #fname_str,
                });
            }
            tmp.#fname = Some(<#ty as #cr::LogixType>::logix_parse(p)?.value)
        ));
        member_tmp_assign.push(quote!(
            tmp.#fname
                .or_else(|| <#ty as #cr::LogixType>::default_value())
                .ok_or_else(|| ParseError::MissingStructMember {
                    span: curly_span.clone(),
                    type_name: #type_name_str,
                    member: #fname_str,
                })?
        ));
        member_tmp_types.push(ty);
        member_names.push(fname);
        member_str_names.push(fname_str);
    }

    (
        quote!(
            #cr::LogixValueDescriptor::Struct {
                members: vec![#(#members_desc,)*],
            }
        ),
        quote!(
            #req_struct_ident
            {
                struct Tmp #impl_gen {
                    #(#member_names: Option<#member_tmp_types>,)*
                }
                let mut tmp = Tmp {
                    #(#member_names: #member_tmp_init,)*
                };

                let mut curly_span = p.req_token(#type_name_str, Token::Brace { start: true, brace: Brace::Curly })?;
                p.req_token(#type_name_str, Token::Newline(false))?;
                'parse_members: loop {
                    match p.next_token()? {
                        #((span, Token::Ident(#member_str_names)) => {
                            p.req_token(#type_name_str, Token::Delim(Delim::Colon))?;
                            #member_tmp_parse;
                            p.req_token(#type_name_str, Token::Newline(false))?;
                        })*
                        (span, Token::Brace { start: false, brace: Brace::Curly }) => {
                            curly_span = span;
                            break 'parse_members;
                        }
                        (span, token) => return Err(ParseError::UnexpectedToken {
                            span,
                            while_parsing: #type_name_str,
                            wanted: Wanted::Tokens(&[
                                Token::Brace { start: false, brace: Brace::Curly },
                                #(Token::Ident(#member_str_names),)*
                            ]),
                            got_token: token.token_type_name(),
                        }),
                    }
                }
                Ok(#cr::Value {
                    value: #prefix #type_name {
                        #(#member_names: #member_tmp_assign,)*
                    },
                    span: type_name_span,
                })
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
        impl_gen: _,
    }: &Shared,
    fields: syn::FieldsUnnamed,
    skip_struct_ident: bool,
) -> (TokenStream2, TokenStream2) {
    let mut members_desc = Vec::new();
    let mut member_indices = Vec::new();
    let mut member_str_names = Vec::new();
    let mut member_parse = Vec::new();

    let req_struct_ident = if skip_struct_ident {
        quote!()
    } else {
        quote!(let type_name_span = p.req_token(#type_name_str, Token::Ident(#type_name_str))?;)
    };

    for (i, field) in fields.unnamed.into_iter().enumerate() {
        let ty = field.ty;
        let fname_str = format!("#{i}");
        members_desc.push(quote!(<#ty as #cr::LogixType>::descriptor()));
        member_parse.push(quote!(<#ty as #cr::LogixType>::logix_parse(p)?.value));
        member_indices.push(i);
        member_str_names.push(fname_str);
    }

    let last_member_parse = member_parse.pop().into_iter();

    (
        quote!(
            #cr::LogixValueDescriptor::Tuple {
                members: vec![#(#members_desc,)*],
            }
        ),
        quote!(
            #req_struct_ident
            p.req_token(#type_name_str, Token::Brace { start: true, brace: Brace::Paren })?;
            Ok(#cr::Value {
                value: #prefix #type_name (
                    #({
                        let value = #member_parse;

                        p.req_token(#type_name_str, Token::Delim(Delim::Comma))?;

                        value
                    },)*
                    #({
                        let value = #last_member_parse;

                        p.req_token(#type_name_str, Token::Brace { start: false, brace: Brace::Paren })?;

                        value
                    },)*
                ),
                span: type_name_span,
            })
        ),
    )
}
