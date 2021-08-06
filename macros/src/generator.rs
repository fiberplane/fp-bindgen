use proc_macro2::TokenStream;
use syn::{ItemEnum, ItemFn, ItemStruct};

pub trait Generator {
    fn generate_deserializable_enum(item: ItemEnum) -> TokenStream;
    fn generate_serializable_enum(item: ItemEnum) -> TokenStream;

    fn generate_deserializable_struct(item: ItemStruct) -> TokenStream;
    fn generate_serializable_struct(item: ItemStruct) -> TokenStream;

    fn generate_import_function(item: ItemFn) -> TokenStream;
    fn generate_export_function(item: ItemFn) -> TokenStream;
}
