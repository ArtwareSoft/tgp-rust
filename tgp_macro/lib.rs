
use proc_macro::TokenStream;
use syn::parse_macro_input;
mod tgp_parse;

#[proc_macro]
pub fn comp(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let res = tgp_parse::comp(input);
    res.unwrap_or_else(|e| e.to_compile_error()).into()
}

#[proc_macro]
pub fn dsl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let res = tgp_parse::dsl(input);
    res.unwrap_or_else(|e| e.to_compile_error()).into()
}

#[proc_macro]
pub fn tgp_value(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let res = tgp_parse::tgp_val(input);
    res.unwrap_or_else(|e| e.to_compile_error()).into()
}

#[proc_macro]
pub fn tgp_value_from_string(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::LitStr);
    let res = tgp_parse::tgp_val_from_string(&input.value());
    res.unwrap_or_else(|e| e.to_compile_error()).into()
}
