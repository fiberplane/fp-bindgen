use proc_macro::TokenStream;

#[proc_macro_derive(Deserialize)]
pub fn derive_deserialize(data_structure: TokenStream) -> TokenStream {
    data_structure
}

#[proc_macro_derive(Serialize)]
pub fn derive_serialize(data_structure: TokenStream) -> TokenStream {
    data_structure
}

#[proc_macro_attribute]
pub fn fp_import(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn fp_export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro]
pub fn fp_bindgen(args: TokenStream) -> TokenStream {
    // TODO
}
