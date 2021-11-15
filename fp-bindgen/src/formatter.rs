use quote::{format_ident, quote, ToTokens};
use syn::token::Async;

use crate::{
    functions::FunctionArg, generators::rust_plugin::format_primitive, types::Type, Function,
};

pub(crate) enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L: ToTokens, R: ToTokens> ToTokens for Either<L, R> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Either::Left(l) => l.to_tokens(tokens),
            Either::Right(r) => r.to_tokens(tokens),
        }
    }
}

pub(crate) struct ExportSafeFunction<'a>(&'a Function);

impl ToTokens for ExportSafeFunction<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Function {
            name,
            args,
            doc_lines,
            is_async,
            return_type,
        } = self.0;

        let name = format_ident!("{}", name);
        let args = args.iter().map(ExportSafeFunctionArg);
        let return_type = if *is_async {
            quote! {FatPtr}
        } else {
            ExportSafeType(return_type).to_token_stream()
        };

        (quote! {
            //#(#doc_lines)*
            fn #name(#(#args),*) -> #return_type
        })
        .to_tokens(tokens);
    }
}

pub struct ExportSafeFunctionArg<'a>(pub &'a FunctionArg);

impl ToTokens for ExportSafeFunctionArg<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = format_ident!("{}", self.0.name);
        let ty = ExportSafeType(&self.0.ty);
        quote!(#name: #ty).to_tokens(tokens)
    }
}

pub struct ExportSafeType<'a>(pub &'a Type);

impl ToTokens for ExportSafeType<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self.0 {
            Type::Primitive(p) => {
                let ty = syn::parse_str::<syn::Type>(&format_primitive(*p)).unwrap();
                quote! {#ty}
            }
            _ => quote! {FatPtr},
        }
        .to_tokens(tokens)
    }
}

#[cfg(test)]
mod test {
    use super::{ExportSafeFunction, ExportSafeFunctionArg};
    use crate::{
        functions::{Function, FunctionArg},
        types::Type,
    };
    use quote::ToTokens;

    #[test]
    fn test_function_arg_to_tokens() {
        let arg = FunctionArg {
            name: "foobar".into(),
            ty: Type::String,
        };
        let arg = ExportSafeFunctionArg(&arg);

        let stringified = arg.into_token_stream().to_string();

        pretty_assertions::assert_eq!(&stringified, "foobar : FatPtr");
    }

    #[test]
    fn test_function_to_tokens() {
        let func = Function {
            name: "foobar".into(),
            is_async: true,
            doc_lines: vec![],
            return_type: Type::String,
            args: vec![FunctionArg {
                name: "a1".into(),
                ty: Type::String,
            }],
        };
        let func = ExportSafeFunction(&func);

        let string = func.into_token_stream().to_string();

        pretty_assertions::assert_eq!(&string, "async fn foobar (a1 : FatPtr) -> FatPtr");
    }
}
