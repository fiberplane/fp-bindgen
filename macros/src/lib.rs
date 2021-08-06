use std::convert::TryFrom;

use proc_macro::TokenStream;
use quote::quote;
use syn::Item;

mod datastructures;
use datastructures::{
    DataStructureItem, DESERIALIZABLE_DATA_STRUCTURES, SERIALIZABLE_DATA_STRUCTURES,
};

mod functions;
use functions::{FunctionItem, FUNCTION_EXPORTS, FUNCTION_IMPORTS};

mod generator;
mod generators;

#[proc_macro_derive(Deserialize)]
pub fn derive_deserialize(item: TokenStream) -> TokenStream {
    let item = DataStructureItem::try_from(syn::parse::<Item>(item).unwrap()).unwrap();

    let ds = DESERIALIZABLE_DATA_STRUCTURES.lock().unwrap();
    ds.insert(item.name(), item);

    TokenStream::new()
}

#[proc_macro_derive(Serialize)]
pub fn derive_serialize(item: TokenStream) -> TokenStream {
    let item = DataStructureItem::try_from(syn::parse::<Item>(item).unwrap()).unwrap();

    let ds = SERIALIZABLE_DATA_STRUCTURES.lock().unwrap();
    ds.insert(item.name(), item);

    TokenStream::new()
}

#[proc_macro_attribute]
pub fn fp_import(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = FunctionItem::try_from(syn::parse::<Item>(item).unwrap()).unwrap();

    let ds = FUNCTION_IMPORTS.lock().unwrap();
    ds.insert(item.name(), item);

    TokenStream::new()
}

#[proc_macro_attribute]
pub fn fp_export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = FunctionItem::try_from(syn::parse::<Item>(item).unwrap()).unwrap();

    let ds = FUNCTION_EXPORTS.lock().unwrap();
    ds.insert(item.name(), item);

    TokenStream::new()
}

#[proc_macro]
pub fn fp_bindgen(_args: TokenStream) -> TokenStream {
    let tokens = TokenStream::new();
    tokens.

    let generator = RustPluginGenerator

    quote!("");
}
