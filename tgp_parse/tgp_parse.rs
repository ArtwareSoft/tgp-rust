use std::convert::TryFrom;
extern crate proc_macro;
use litrs::Literal;
extern crate paste;
use proc_macro2::{Ident, TokenStream, TokenTree};
use proc_macro2::Delimiter::{Brace, Bracket, Parenthesis};
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
    let cloned_body = body.clone();
    let mut iter = body.into_iter();
    let tt = iter.next().unwrap();
    match tt {
        TokenTree::Literal(_) => literal_value(&tt),
        TokenTree::Ident(pt) => match iter.next().unwrap() {
            TokenTree::Group(g) => match g.delimiter() {
                Brace => build_profile(&pt.to_string() ,g.stream()),
                Parenthesis => build_profile_by_value(&pt.to_string() ,g.stream()),
                Bracket => return Err(Error::new(g.span(), "expecting profile body")),
                proc_macro2::Delimiter::None => return Err(Error::new(g.span(), "expecting profile body")),
            },
            TokenTree::Literal(l) => return Err(Error::new(l.span(), "expecting profile body. use {")),
            TokenTree::Ident(i) => return Err(Error::new(i.span(), "expecting profile body. use {")),
            TokenTree::Punct(p) => return Err(Error::new(p.span(), "expecting profile body. use {")),
        },
        TokenTree::Group(g) => match g.delimiter() {
            Brace => build_profile("$obj" ,g.stream()),
            Parenthesis => return Err(Error::new(g.span(), "functions are not supported")), // build_function("", cloned_body),
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
    let mut props: Vec<(Ident, TokenStream)> = Vec::new();
    let mut iter = body.into_iter();
    while let Some(tt) = iter.next()  {
        match tt {
            TokenTree::Ident(att_name) => {
                match iter.next().unwrap() {
                    TokenTree::Group(g) => return Err(Error::new(g.span(), "expecting COLON")),
                    TokenTree::Literal(l) => return Err(Error::new(l.span(), "expecting COLON")),
                    TokenTree::Ident(i) => return Err(Error::new(i.span(), "expecting COLON")),
                    TokenTree::Punct(p) => match p.as_char() {
                        ':' => {},
                        ',' => { if props.len() == 0 { 
                                    props.push((Ident::new("id", att_name.span()), att_name.to_token_stream())) 
                                } else { return Err(Error::new(p.span(), "expecting COLON"))}
                            },
                        _ => return Err(Error::new(p.span(), "expecting COLON"))
                    }
                }

                let v = iter.next().unwrap();
                match v {
                    TokenTree::Literal(_) => match literal_value(&v) { 
                        Err(e) => return Err(e),
                        Ok(ts) => props.push((att_name, ts))
                    },
                    TokenTree::Ident(pt) => match iter.next().unwrap() {
                        TokenTree::Group(g) => {
                            match g.delimiter() {
                                Brace => match build_profile(&pt.to_string() ,g.stream()) {
                                    Err(e) => return Err(e),
                                    Ok(ts) => props.push((att_name, ts))
                                },
                                Parenthesis => match build_profile_by_value(&pt.to_string() ,g.stream()) {
                                    Err(e) => return Err(e),
                                    Ok(ts) => props.push((att_name, ts))
                                },                                
                                Bracket => return Err(Error::new(g.span(), "expecting profile body")),
                                proc_macro2::Delimiter::None => return Err(Error::new(g.span(), "expecting profile body")),
                            }
                        },
                        TokenTree::Punct(p) => match p.as_char() {
                            ',' => props.push((att_name, quote! {{ TgpValue::String(stringify!(#pt))}})),
                            _ => return Err(Error::new(p.span(), "expecting profile body"))
                        }
                        _ => return Err(Error::new(pt.span(), "expecting profile body"))
                    },
                    TokenTree::Group(g) => {
                        match g.delimiter() {
                            Bracket => match tgp_array(g.stream()) {
                                Err(e) => return Err(e),
                                Ok(ts) => props.push((att_name, ts))
                            },
                            Brace => return Err(Error::new(g.span(), "as is Object is not supported yet")),
                            Parenthesis => return Err(Error::new(g.span(), "functions are not suppoted yet")),
                            proc_macro2::Delimiter::None => return Err(Error::new(g.span(), "expecting profile body")),
                        }                
                    },        
                    TokenTree::Punct(p) => return Err(Error::new(p.span(), "expecting tgp value"))
                }
            },
            TokenTree::Literal(l) => return Err(Error::new(l.span(), "expecting prop by name")),
            TokenTree::Group(g) => return Err(Error::new(g.span(), "mixed values are not supported yet")),
            TokenTree::Punct(p) => if p.as_char() != ',' { return Err(Error::new(p.span(), "expecting COMMA")) },
        }
    }

    let hashmap_entries = props.iter().map(|(key, value)| { quote! { map.insert( stringify!(#key), #value); } });
    
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
    let mut array: Vec<TokenStream> = Vec::new();
    let mut iter = body.into_iter();
    while let Some(tt) = iter.next()  {
        match tt {
            TokenTree::Literal(_) => {
                match literal_value(&tt) { 
                    Err(e) => return Err(e),
                    Ok(ts) => array.push(ts)
                }
            },
            TokenTree::Ident(pt) => match iter.next().unwrap() {
                TokenTree::Group(g) => match g.delimiter() {
                    Brace => match build_profile(&pt.to_string() ,g.stream()) {
                        Err(e) => return Err(e),
                        Ok(ts) => array.push(ts)
                    },
                    Parenthesis => match build_profile_by_value(&pt.to_string() ,g.stream()) {
                        Err(e) => return Err(e),
                        Ok(ts) => array.push(ts)
                    },
                    Bracket => return Err(Error::new(g.span(), "expecting profile body")),
                    proc_macro2::Delimiter::None => return Err(Error::new(g.span(), "expecting profile body")),
                },
                TokenTree::Literal(l) => return Err(Error::new(l.span(), "expecting profile body. use {")),
                TokenTree::Ident(i) => return Err(Error::new(i.span(), "expecting profile body. use {")),
                TokenTree::Punct(p) => return Err(Error::new(p.span(), "expecting profile body. use {")),
            },
            TokenTree::Group(g) => match g.delimiter() {
                Brace => match build_profile("$obj" ,g.stream()) {
                    Err(e) => return Err(e),
                    Ok(ts) => array.push(ts)
                },
                Parenthesis => return Err(Error::new(g.span(), "functions in array are not supported yet")),
                Bracket => return Err(Error::new(g.span(), "nested arrays are not supported yet")),
                proc_macro2::Delimiter::None => return Err(Error::new(g.span(), "expecting tgp value in array")),
            },
            TokenTree::Punct(p) => if p.as_char() != ',' { return Err(Error::new(p.span(), "expecting COMMA")) },
        }
    }

    let vec_items = array.iter().map(|value| { quote! { vec.push(#value); } });
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

// fn build_function(id: &str, body: TokenStream) -> Result<TokenStream> {
//     let mut header: TokenStream;
//     let mut body: TokenStream;
//     let mut iter = body.into_iter();
//     match iter.next().unwrap() {
//         TokenTree::Literal(_) => return Err(Error::new(g.span(), "expecting func header")),
//         TokenTree::Ident(id) => match iter.next().unwrap() {
//             TokenTree::Group(g) => match g.delimiter() {
//                 Brace => return Err(Error::new(g.span(), "expecting (")),
//                 Parenthesis => build_function(&id.to_string(), g.stream()),
//                 Bracket => return Err(Error::new(g.span(), "expecting (")),
//                 proc_macro2::Delimiter::None => return Err(Error::new(g.span(), "expecting profile body")),
//             },
//             TokenTree::Literal(l) => return Err(Error::new(l.span(), "expecting profile body. use {")),
//             TokenTree::Ident(i) => return Err(Error::new(i.span(), "expecting profile body. use {")),
//             TokenTree::Punct(p) => return Err(Error::new(p.span(), "expecting profile body. use {")),
//         },
//         TokenTree::Group(g) => match g.delimiter() {
//             Brace => match build_profile("$obj" ,g.stream()) {
//                 Err(e) => return Err(e),
//                 Ok(ts) => array.push(ts)
//             },
//             Parenthesis => return Err(Error::new(g.span(), "functions in array are not supported yet")),
//             Bracket => return Err(Error::new(g.span(), "nested arrays are not supported yet")),
//             proc_macro2::Delimiter::None => return Err(Error::new(g.span(), "expecting tgp value in array")),
//         },
//         TokenTree::Punct(p) => if p.as_char() != ',' { return Err(Error::new(p.span(), "expecting COMMA")) },
//     }
    

//     let vec_items = array.iter().map(|value| { quote! { vec.push(#value); } });
//     let res = quote! {
//         {
//             let mut vec: Vec<TgpValue> = Vec::new();
//             #(#vec_items)*
//             TgpValue::Array(vec)
//         }
//     };
//     println!("vec_items {}",res.to_string());
//     Ok(res)
// }

fn build_profile_by_value(pt: &str, body: TokenStream) -> Result<TokenStream> {
    let mut array: Vec<TokenStream> = Vec::new();
    let mut iter = body.into_iter();
    while let Some(tt) = iter.next()  {
        match tt {
            TokenTree::Literal(_) => {
                match literal_value(&tt) { 
                    Err(e) => return Err(e),
                    Ok(ts) => array.push(ts)
                }
            },
            TokenTree::Ident(pt) => match iter.next().unwrap() {
                TokenTree::Group(g) => match g.delimiter() {
                    Brace => match build_profile(&pt.to_string() ,g.stream()) {
                        Err(e) => return Err(e),
                        Ok(ts) => array.push(ts)
                    },
                    Parenthesis => match build_profile_by_value(&pt.to_string() ,g.stream()) {
                        Err(e) => return Err(e),
                        Ok(ts) => array.push(ts)
                    },
                    Bracket => return Err(Error::new(g.span(), "expecting profile body")),
                    proc_macro2::Delimiter::None => return Err(Error::new(g.span(), "expecting profile body")),
                },
                TokenTree::Literal(l) => return Err(Error::new(l.span(), "expecting profile body. use {")),
                TokenTree::Ident(i) => return Err(Error::new(i.span(), "expecting profile body. use {")),
                TokenTree::Punct(p) => return Err(Error::new(p.span(), "expecting profile body. use {")),
            },
            TokenTree::Group(g) => match g.delimiter() {
                Brace => match build_profile("$mixed" ,g.stream()) {
                    Err(e) => return Err(e),
                    Ok(ts) => array.push(ts)
                },
                Parenthesis => return Err(Error::new(g.span(), "functions are not supported yet")),
                Bracket => return Err(Error::new(g.span(), "use list() instead of []")),
                proc_macro2::Delimiter::None => return Err(Error::new(g.span(), "expecting profile value")),
            },
            TokenTree::Punct(p) => if p.as_char() != ',' { return Err(Error::new(p.span(), "expecting COMMA")) },
        }
    }

    let vec_items = array.iter().map(|value| { quote! { vec.push(#value); } });
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