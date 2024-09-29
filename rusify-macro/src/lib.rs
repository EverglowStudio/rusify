extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemEnum, ItemStruct, ItemFn};

#[proc_macro_attribute]
pub fn rusify_enum(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemEnum);
    let expanded = quote! {
        #[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
        #[cfg_attr(target_arch = "wasm32", derive(tsify_next::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
        #[cfg_attr(target_env = "ohos", napi_derive_ohos::napi)]
        #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        #input
    };
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn rusify_struct(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let expanded = quote! {
        #[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
        #[cfg_attr(target_arch = "wasm32", derive(tsify_next::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
        #[cfg_attr(target_env = "ohos", napi_derive_ohos::napi(object))]
        #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        #input
    };
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn rusify_export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let expanded = quote! {
        #[cfg_attr(feature = "uniffi", uniffi::export)]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
        #[cfg_attr(target_env = "ohos", napi_derive_ohos::napi)]
        #input
    };
    TokenStream::from(expanded)
}

#[proc_macro]
pub fn scaffolding(_item: TokenStream) -> TokenStream {
    let expanded = quote! {
        #[cfg(feature = "uniffi")]
        uniffi::setup_scaffolding!();
    };
    TokenStream::from(expanded)
}