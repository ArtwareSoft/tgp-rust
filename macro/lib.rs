
use proc_macro::TokenStream;
use syn::{parse_macro_input};

mod tgp_macro;

#[proc_macro]
pub fn comp(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let res = tgp_macro::tgp_val(input);
    res.unwrap_or_else(|e| e.to_compile_error()).into()
}

#[proc_macro]
pub fn tgp_value(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let res = tgp_macro::tgp_val(input);
    res.unwrap_or_else(|e| e.to_compile_error()).into()
}

#[proc_macro]
pub fn tgp_val_from_string(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::LitStr);
    let res = tgp_macro::tgp_val_from_string(&input.value());
    res.unwrap_or_else(|e| e.to_compile_error()).into()
}
