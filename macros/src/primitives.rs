use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenTree};
use quote::{quote, ToTokens};
use std::str::FromStr;
use syn::Type;

/// Type of primitive that is supported out-of-the-box.
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
    pub fn gen_impl(&self) -> TokenStream {
        let ty = self.ty();
        let ty_str = ty.to_token_stream().to_string();

        let implementation = quote! {
            impl Serializable for #ty {
                fn ident() -> TypeIdent {
                    TypeIdent::from(#ty_str)
                }

                fn is_primitive() -> bool {
                    true
                }

                fn ty() -> Type {
                    Type::Primitive(Primitive::#self)
                }
            }

            impl<const N: usize> Serializable for [#ty; N] {
                fn ident() -> TypeIdent {
                    TypeIdent::from(format!("[{}; {}]", #ty_str, N).as_str())
                }

                fn ty() -> Type {
                    Type::Array(Primitive::#self, N)
                }
            }
        };
        implementation.into()
    }

    fn ty(&self) -> Type {
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
        Type::Path(parse_str(string))
    }
}

impl ToTokens for Primitive {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use Primitive::*;
        let ident_str = match self {
            Bool => "Bool",
            F32 => "F32",
            F64 => "F64",
            I8 => "I8",
            I16 => "I16",
            I32 => "I32",
            I64 => "I64",
            U8 => "U8",
            U16 => "U16",
            U32 => "U32",
            U64 => "U64",
        };
        let ident = Ident::new(ident_str, Span::call_site());
        tokens.extend(vec![TokenTree::Ident(ident)].into_iter());
    }
}

fn parse_str<T: syn::parse::Parse>(string: &str) -> T {
    syn::parse::<T>(TokenStream::from_str(string).unwrap()).unwrap()
}
