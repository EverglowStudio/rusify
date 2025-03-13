extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemEnum, ItemFn, ItemStruct, ItemTrait};

#[proc_macro_attribute]
pub fn rusify_enum(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemEnum);
    let expanded = quote! {
        #[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
        #[cfg_attr(feature = "wasm", derive(tsify_next::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
        #[cfg_attr(feature = "ohos", napi_derive_ohos::napi)]
        #input
    };
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn rusify_struct(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let expanded = quote! {
        #[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
        #[cfg_attr(feature = "wasm", derive(tsify_next::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
        #[cfg_attr(feature = "ohos", napi_derive_ohos::napi(object))]
        #input
    };
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn rusify_interface(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemTrait);
    let expanded = quote! {
        #[cfg_attr(feature = "uniffi", uniffi::export(callback_interface))]
        #input
    };
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn rusify_export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let expanded = quote! {
        #[cfg_attr(feature = "uniffi", uniffi::export)]
        #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
        #[cfg_attr(feature = "ohos", napi_derive_ohos::napi)]
        #input
    };
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn rusify_export_async_uniffi(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let expanded = quote! {
        #[cfg(feature = "uniffi")]
        #[uniffi::export(async_runtime = "tokio")]
        #input
    };
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn rusify_export_async_ohos(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let expanded = quote! {
        #[cfg(feature = "ohos")]
        #[napi_derive_ohos::napi]
        #input
    };
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn rusify_export_async(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let expanded = quote! {
        #[cfg_attr(feature = "uniffi", uniffi::export(async_runtime = "tokio"))]
        #[cfg_attr(feature = "ohos", napi_derive_ohos::napi)]
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

#[proc_macro_attribute]
pub fn rusify_object(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let expanded = quote! {
        #[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
        #[cfg_attr(feature = "wasm", derive(tsify_next::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
        #[cfg_attr(feature = "ohos", napi_derive_ohos::napi(object))]
        #[derive(Clone)]
        #input
    };
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn rusify_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemImpl);
    let expanded = quote! {
        #[cfg_attr(feature = "uniffi", uniffi::export)]
        #input
    };
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn rusify_constructor(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let expanded = quote! {
        #[cfg_attr(feature = "uniffi", uniffi::constructor)]
        #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
        #[cfg_attr(feature = "ohos", napi_derive_ohos::napi)]
        #input
    };
    TokenStream::from(expanded)
}
