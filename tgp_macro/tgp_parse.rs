use std::convert::TryFrom;
extern crate proc_macro;
use litrs::Literal;
use proc_macro2::{TokenStream, TokenTree};
use proc_macro2::Delimiter::{Brace, Bracket, Parenthesis};
use syn::spanned::Spanned;
use syn::{Result, Error};
use quote::{format_ident, quote};

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
    let span = body.span();
    let mut iter = body.into_iter();
    let ident = match iter.next() {
        Some(TokenTree::Ident(iden)) => iden,
        _ => return Err(Error::new(span, "expecting comp name"))
    };
    let cmp = match iter.next() {
        Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => tgp_val(iter.collect()),
        Some(tt) => Err(Error::new(tt.span(), "expecting , 2")),
        None => Err(Error::new(ident.span(), "expecting , 3"))
    }?;
    let fn_name = format_ident!("{}_init", ident);
    let res = quote! { 
        #[ctor]
        #[allow(non_snake_case)]
        fn #fn_name() {
            COMPS.add(module_path!(), stringify!(#ident), #cmp) 
        }
    };
    println!("comp {}",res.to_string());
    Ok(res)
}

pub fn dsl(dsl: TokenStream) -> Result<TokenStream> {
    let span = dsl.span();
    match dsl.into_iter().next() {
        Some(TokenTree::Ident(iden)) => Ok(quote! {{DSLs.add(module_path!(), stringify!(#iden)) }}),
        _ => Err(Error::new(span, "expecting dsl"))
    }
}

fn tgp_function(func: TokenStream) -> Result<TokenStream> {
    let span = func.span();
    let mut iter = func.into_iter();
    match iter.next() {
        Some(TokenTree::Punct(p)) if p.to_string() == "<" => match iter.next() {
            Some(TokenTree::Ident(res_type)) => { // fn<Exp> |profile: &'static Profile| { ...
                match iter.next() {
                    Some(TokenTree::Punct(p)) if p.as_char() == '>' => {},
                    _ => return Err(Error::new(res_type.span(), "expecting >"))
                }
            
                let body: TokenStream = iter.collect();
                return Ok(quote! {{
                    TgpValue::RustImpl(Arc::new(Arc::new(#body) as FuncType<#res_type>))
                }})        
            },
            _ => return Err(Error::new(p.span(), "expecting <TYPE>"))
        }
        Some(TokenTree::Group(g)) if g.delimiter() == Parenthesis => { // fn (x: Fn Exp, y: Exp) -> Exp { x() + y },
        // => |profile: &'static Profile| {
        //    match (profile.prop::<Exp>("x"), profile.prop::<Exp>("y")) {
        //        (x, y) => { x + y }
        //    }
        // }
            let params_in_match_exp = join_token_streams_with_comma(split_token_stream(g.stream(), ',')
                .map(|x| param_to_match_exp(x).unwrap()).collect());
            let param_names = join_token_streams_with_comma(split_token_stream(g.stream(), ',')
                .map(|x| param_name(x).unwrap()).collect());

            let arrow = match iter.next() {
                Some(TokenTree::Punct(p)) if p.as_char() == '-' => match iter.next() {
                    Some(TokenTree::Punct(p)) if p.as_char() == '>' => {p},
                    _ => return Err(Error::new(p.span(), "expecting >"))
                },
                _ => return Err(Error::new(g.span(), "expecting -> at the end"))
            };
            let res_type = match iter.next() {
                Some(TokenTree::Ident(res_type)) => res_type,
                _ => return Err(Error::new(arrow.span(), "expecting result type"))
            };
            let body: TokenStream = iter.collect();
            return Ok(quote! {{
                TgpValue::RustImpl(Arc::new(Arc::new(
                    |profile: &'static Profile| {
                        match (#params_in_match_exp) {
                            (#param_names) => #body
                        }
                    }) as FuncType<#res_type>))
            }})
        },
        _ => return Err(Error::new(span, "expecting function def:  fn(x: Exp, y: fn Exp) | fn<T> |profile: &'static Profile|"))
    }

    fn join_token_streams_with_comma(streams: Vec<TokenStream>) -> TokenStream {
        let mut iter = streams.into_iter();
        if let Some(first) = iter.next() {
            iter.fold(first, |acc, ts| quote! { #acc , #ts })
        } else {
            TokenStream::new()
        }
    }
    
    fn param_name(param: TokenStream) -> Result<TokenStream> {
        let span = param.span();
        let mut iter = param.into_iter();
        match iter.next() {
            Some(TokenTree::Ident(param)) => Ok(quote!{#param}),
            _ => return Err(Error::new(span, "expecting param identifier"))
        }
    }    

    fn param_to_match_exp(param: TokenStream) -> Result<TokenStream> {
        let span = param.span();
        let mut iter = param.into_iter();
        let iden = match iter.next() {
            Some(TokenTree::Ident(param)) => param,
            _ => return Err(Error::new(span, "expecting param identifier"))
        };
        let colon = match iter.next() {
            Some(TokenTree::Punct(p)) if p.as_char() == ':' => { p },
            _ => return Err(Error::new(iden.span(), "expecting :"))
        };
        match iter.next() {
            Some(TokenTree::Ident(t)) if t.to_string() == "fn" => match iter.next() {
                Some(TokenTree::Ident(t)) => { Ok(quote! {profile.func::<#t>(stringify!(#iden)) }) },
                _ => return Err(Error::new(t.span(), "expecting param type identifier"))
            },
            Some(TokenTree::Ident(t)) => { Ok(quote! {profile.prop::<#t>(stringify!(#iden)) }) },
            _ => return Err(Error::new(colon.span(), "expecting param type identifier"))
        }
    }    
}

pub fn tgp_val(body: TokenStream) -> Result<TokenStream> {
    let span = body.span();
    let mut iter = body.into_iter();
    let tt = match iter.next() {
        Some(tt) => tt,
        None => return Err(Error::new(span, "expecting tgp value 1"))
    };
    match tt {
        TokenTree::Literal(_) => literal_value(&tt),
        TokenTree::Ident(func) if func.to_string() == "fn" => tgp_function(iter.collect()),
        TokenTree::Ident(pt) => match iter.next() {
            None => return Ok(quote! {TgpValue::Iden(stringify!(#pt)) }),
            Some(TokenTree::Group(g)) => match g.delimiter() {
                Brace => build_profile(&pt.to_string() ,g.stream()),
                Parenthesis => build_profile_by_value(&pt.to_string() ,g.stream()),
                Bracket => return Err(Error::new(g.span(), "expecting profile body 1 ")),
                proc_macro2::Delimiter::None => return Err(Error::new(g.span(), "expecting profile body 2")),
            },
            Some(TokenTree::Literal(l)) => return Err(Error::new(l.span(), "expecting profile body 3. use (")),
            // Some(TokenTree::Punct(p)) if p.as_char() == '<' => {
            //     match pt.to_string().as_str() {
            //     "fn" => {
            //         let type_iden = match iter.next() {
            //         Some(TokenTree::Ident(type_iden)) => type_iden,
            //     _ => return Err(Error::new(p.span(), "expecting type identifier"))
            // };
            // match iter.next() {
            //     Some(TokenTree::Punct(p)) if p.as_char() == '>' => {},
            //     _ => return Err(Error::new(p.span(), "expecting >"))
            // }
        
            // let body: TokenStream = iter.collect();
            // return Ok(quote! {{
            // TgpValue::RustImpl(Arc::new(Arc::new(#body) as FuncType<#type_iden>))
            // }})        
            // }
            //         _ => return Err(Error::new(p.span(), "expecting fn before <"))
            // }
            // },
            _ => return Err(Error::new(pt.span(), "expecting profile body 5. use (")),
        },
        TokenTree::Group(g) => match g.delimiter() {
            Brace => build_profile("$obj" ,g.stream()),
            Parenthesis => build_function(g.stream(), iter.collect()),
            Bracket => tgp_array(g.stream()),
            proc_macro2::Delimiter::None => return Err(Error::new(g.span(), "expecting array [")),
        }
        // TokenTree::Punct(p) if p.as_char() == '|' => {
        // let body: TokenStream = iter.collect();
        // return Ok(quote! {{
        // TgpValue::RustImpl(Arc::new(| #body))
        // }})
        // }
        TokenTree::Punct(p) => return Err(Error::new(p.span(), "expecting tgp value 2"))
    }
}

fn literal_value(input: &TokenTree) -> Result<TokenStream> {
    match Literal::try_from(input) {
        Err(_) => {
            println!("error1");
            return Err(Error::new(input.span(), "invalid literal"))
        },
        Ok(Literal::Integer(_)) => Ok(quote! {{ TgpValue::Int(#input) }}),
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
            TgpValue::UnresolvedProfile(#pt,vec)
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