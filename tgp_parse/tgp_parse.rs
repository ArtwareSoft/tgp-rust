use std::convert::TryFrom;
extern crate proc_macro;
use litrs::Literal;
extern crate paste;
use proc_macro2::{TokenStream, TokenTree};
use proc_macro2::Delimiter::{Brace, Bracket, Parenthesis};
use syn::spanned::Spanned;
use syn::{Result, Error};
use quote::{quote, ToTokens};

pub fn tgp_val_from_string(body: &str) -> Result<TokenStream> {
    let mut fixed = String::new();
    let mut in_single_quotes = false;
    let mut chars = body.chars();

    while let Some(c) = chars.next() {
        match c {
            '\'' if in_single_quotes => {
                in_single_quotes = false;
                fixed.push('"');
            },
            '\'' if !in_single_quotes => {
                in_single_quotes = true;
                fixed.push('"');
            },
            '"' if in_single_quotes => { fixed.push_str("\\\"") },
            _ => { fixed.push(c) }
        }
    }
    tgp_val(fixed.parse().unwrap())
}

pub fn comp(body: TokenStream) -> Result<TokenStream> {
    tgp_val(quote!{component(#body)})
}

pub fn tgp_val(body: TokenStream) -> Result<TokenStream> {
    let span = body.span();
    let mut iter = body.into_iter();
    let tt = match iter.next() {
        Some(tt) => tt,
        None => return Err(Error::new(span, "expecting tgp value"))
    };
    match tt {
        TokenTree::Literal(_) => literal_value(&tt),
        TokenTree::Ident(pt) => match iter.next() {
            None => return Ok(quote! {TgpValue::Iden(stringify!(#pt)) }),
            Some(TokenTree::Group(g)) => match g.delimiter() {
                Brace => build_profile(&pt.to_string() ,g.stream()),
                Parenthesis => build_profile_by_value(&pt.to_string() ,g.stream()),
                Bracket => return Err(Error::new(g.span(), "expecting profile body")),
                proc_macro2::Delimiter::None => return Err(Error::new(g.span(), "expecting profile body")),
            },
            Some(TokenTree::Literal(l)) => return Err(Error::new(l.span(), "expecting profile body. use (")),
            Some(TokenTree::Ident(i)) => if pt.to_string() == "fn" {
                    let body: TokenStream = iter.collect();
                    return Ok(quote! {{
                        #[derive(Debug)]
                        struct tmp;
                        impl RustImpl for tmp {
                            fn run #body
                        }
                        TgpValue::RustImpl(Arc::new(tmp)) 
                    }})
                } else {
                    return Err(Error::new(i.span(), "expecting profile body. use ("))
            },
            Some(TokenTree::Punct(p)) => return Err(Error::new(p.span(), "expecting profile body. use (")),
        },
        TokenTree::Group(g) => match g.delimiter() {
            Brace => build_profile("$obj" ,g.stream()),
            Parenthesis => build_function(g.stream(), iter.collect()),
            Bracket => tgp_array(g.stream()),
            proc_macro2::Delimiter::None => return Err(Error::new(g.span(), "expecting array [")),
        }
        TokenTree::Punct(p) => return Err(Error::new(p.span(), "expecting tgp value"))
    }
}

fn literal_value(input: &TokenTree) -> Result<TokenStream> {
    match Literal::try_from(input) {
        Err(_) => {
            println!("error1");
            return Err(Error::new(input.span(), "invalid literal"))
        },
        Ok(Literal::Integer(_)) => Ok(quote! {{ TgpValue::I32(#input) }}),
        Ok(Literal::Bool(_)) => Ok(quote! {{ TgpValue::Boolean(#input)}}),
        Ok(_) => Ok(quote! {{ TgpValue::String(#input)}})
    }
}

fn build_profile(pt: &str, body: TokenStream) -> Result<TokenStream> {
    let span = body.span();
    let hashmap_entries = split_token_stream(body, ',').try_fold(vec![], |mut acc, att_val| {
        let mut iter = att_val.into_iter();
        let iden = match iter.next() {
            Some(TokenTree::Ident(i)) => i,
            _ => return Err(Error::new(span, "expecting iden"))
        };
        match iter.next() {
            Some(TokenTree::Punct(punct)) => match punct.as_char() {
                ':' => punct,
                _ => return Err(Error::new(punct.span(), "expecting colon")),
            },
            Some(tt) => return Err(Error::new(tt.span(), "expecting colon")),
            None => return Err(Error::new(iden.span(), "expecting colon after iden"))
        };        
        let tgp_value = match tgp_val(iter.collect()) {
            Ok(tt) => tt,
            Err(e) => return Err(e)
        };
        acc.push(quote! { map.insert(stringify!(#iden), #tgp_value); });
        Ok(acc)
    })?;
    let res = quote! {
        {
            let mut map = std::collections::HashMap::new();
            #(#hashmap_entries)*
            Profile::new(#pt, map)
        }
    };
    println!("obj_props {}",res.to_string());
    Ok(res)
}

fn tgp_array(body: TokenStream) -> Result<TokenStream> {
    let vec_items = split_token_stream(body, ',').try_fold(vec![], |mut acc, val| {
        let tgp_value = match tgp_val(val) {
            Ok(tt) => tt,
            Err(e) => return Err(e)
        };
        acc.push(quote! { vec.push(#tgp_value); });
        Ok(acc)
    })?;
    let res = quote! {
        {
            let mut vec: Vec<TgpValue> = Vec::new();
            #(#vec_items)*
            TgpValue::Array(vec)
        }
    };
    println!("vec_items {}",res.to_string());
    Ok(res)
}

fn build_profile_by_value(pt: &str, body: TokenStream) -> Result<TokenStream> {
    let vec_items = split_token_stream(body, ',').try_fold(vec![], |mut acc, val| {
        let tgp_value = match tgp_val(val) {
            Ok(tt) => tt,
            Err(e) => return Err(e)
        };
        acc.push(quote! { vec.push(#tgp_value); });
        Ok(acc)
    })?;
    
    let res = quote! {
        {
            let mut vec: Vec<TgpValue> = Vec::new();
            #(#vec_items)*
            TgpValue::ProfileByValue(#pt,vec)
        }
    };
    println!("profile_by_value {}",res.to_string());
    Ok(res)
}

fn build_function(header: TokenStream, rest: TokenStream) -> Result<TokenStream> {    
    let mut iter = rest.into_iter();
    match iter.next() {
        Some(TokenTree::Punct(punct)) => match punct.as_char() {
            '=' => punct,
            _ => return Err(Error::new(header.span(), "expecting =>")),
        },
        Some(tt) => return Err(Error::new(tt.span(), "expecting =>")),
        None => return Err(Error::new(header.span(), "expecting =>"))
    };
    let sep = match iter.next() {
        Some(TokenTree::Punct(punct)) => match punct.as_char() {
            '>' => punct,
            _ => return Err(Error::new(header.span(), "expecting =>")),
        },
        Some(tt) => return Err(Error::new(tt.span(), "expecting =>")),
        None => return Err(Error::new(header.span(), "expecting =>"))
    };
    let body = match iter.next() {
        Some(TokenTree::Group(g)) => g,
        Some(tt) => return Err(Error::new(tt.span(), "expecting func body")),
        None => return Err(Error::new(sep.span(), "expecting func body"))
    };
    let res = quote! {
        {
            let st =  as_static(&format!("{} {}", stringify!(#header), stringify!(#body)));
            TgpValue::JsFunc(st)
        }
    };
    println!("profile_by_value {}",res.to_string());
    Ok(res)
}

use std::iter::from_fn;
fn split_token_stream(input: TokenStream, delimiter: char) -> impl Iterator<Item = TokenStream> {
    let mut iter = input.into_iter().peekable();
    from_fn(move || {
        let mut segment = Vec::new();
        while let Some(token) = iter.next() {
            match &token {
                TokenTree::Punct(punct) if punct.as_char() == delimiter => {
                    if !segment.is_empty() {
                        return Some(TokenStream::from_iter(segment.into_iter()));
                    }
                    segment = Vec::new(); // Clear the segment after processing
                },
                _ => segment.push(token),
            }
        }

        if !segment.is_empty() {
            Some(TokenStream::from_iter(segment.into_iter()))
        } else {
            None
        }
    })
}