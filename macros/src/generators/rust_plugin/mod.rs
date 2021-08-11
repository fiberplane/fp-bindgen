use crate::generator::Generator;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{ItemEnum, ItemFn, ItemStruct};

struct RustPluginGenerator {}

impl Generator for RustPluginGenerator {
    fn generate_serializable_enum(item: ItemEnum) -> TokenStream {
        item.into_token_stream()
    }

    fn generate_deserializable_enum(item: ItemEnum) -> TokenStream {
        item.into_token_stream()
    }

    fn generate_serializable_struct(item: ItemStruct) -> TokenStream {
        item.into_token_stream()
    }

    fn generate_deserializable_struct(item: ItemStruct) -> TokenStream {
        item.into_token_stream()
    }

    fn generate_import_function(item: ItemFn) -> TokenStream {
        item.into_token_stream()
    }

    fn generate_export_function(item: ItemFn) -> TokenStream {
        item.into_token_stream()
    }
}
