use quote::{quote, ToTokens};
use std::str::FromStr;

/// Type of primitive that is supported out-of-the-box.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Primitive {
    Bool,
    F32,
    F64,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
}

impl Primitive {
    pub fn name(&self) -> String {
        use Primitive::*;
        let string = match self {
            Bool => "bool",
            F32 => "f32",
            F64 => "f64",
            I8 => "i8",
            I16 => "i16",
            I32 => "i32",
            I64 => "i64",
            U8 => "u8",
            U16 => "u16",
            U32 => "u32",
            U64 => "u64",
        };
        string.to_owned()
    }

    pub fn js_array_name(&self) -> Option<String> {
        use Primitive::*;
        match self {
            U8 => Some("Uint8Array"),
            U16 => Some("Uint16Array"),
            U32 => Some("Uint32Array"),
            I8 => Some("Int8Array"),
            I16 => Some("Int16Array"),
            I32 => Some("Int32Array"),
            F32 => Some("Float32Array"),
            F64 => Some("Float64Array"),
            _ => None,
        }
        .map(|s| s.to_owned())
    }
}

impl FromStr for Primitive {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let primitive = match string {
            "bool" => Primitive::Bool,
            "f32" => Primitive::F32,
            "f64" => Primitive::F64,
            "i8" => Primitive::I8,
            "i16" => Primitive::I16,
            "i32" => Primitive::I32,
            "i64" => Primitive::I64,
            "u8" => Primitive::U8,
            "u16" => Primitive::U16,
            "u32" => Primitive::U32,
            "u64" => Primitive::U64,
            string => return Err(format!("Unknown primitive type: \"{string}\"")),
        };
        Ok(primitive)
    }
}

impl ToTokens for Primitive {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        (match self {
            Primitive::Bool => quote! {bool},
            Primitive::F32 => quote! {f32},
            Primitive::F64 => quote! {f64},
            Primitive::I8 => quote! {i8},
            Primitive::I16 => quote! {i16},
            Primitive::I32 => quote! {i32},
            Primitive::I64 => quote! {i64},
            Primitive::U8 => quote! {u8},
            Primitive::U16 => quote! {u16},
            Primitive::U32 => quote! {u32},
            Primitive::U64 => quote! {u64},
        })
        .to_tokens(tokens)
    }
}
