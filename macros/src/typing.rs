use once_cell::sync::Lazy;
use proc_macro::TokenStream;
use proc_macro_error::{abort, emit_error, proc_macro_error};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use std::{cmp::max, collections::HashMap, convert::TryInto, sync::RwLock};
use syn::{
    parse::Parse, spanned::Spanned, FnArg, ForeignItemFn, ItemFn, PatType, ReturnType, Signature,
    Type,
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct InputSignature {
    name: String,
    ty: String,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct FnSignature {
    pub is_const: bool,
    pub is_async: bool,
    pub name: String,
    pub inputs: Vec<InputSignature>,
    pub output: String,
}

impl<'a> From<&'a Signature> for FnSignature {
    fn from(sig: &'a Signature) -> Self {
        Self {
            is_async: sig.asyncness.is_some(),
            is_const: sig.constness.is_some(),
            name: sig.ident.to_string(),
            output: (match &sig.output {
                syn::ReturnType::Default => "()".to_owned(),
                syn::ReturnType::Type(_, ty) => ty.to_token_stream().to_string(),
            }),
            inputs: sig
                .inputs
                .iter()
                .map(|arg| {
                    let pt = get_type(arg);
                    InputSignature {
                        name: pt.pat.as_ref().to_token_stream().to_string(),
                        ty: pt.ty.as_ref().to_token_stream().to_string(),
                    }
                })
                .collect::<Vec<_>>(),
        }
    }
}

impl Parse for FnSignature {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let sig = ForeignItemFn::parse(input)
            .map(|i| i.sig)
            .or_else(|_| ItemFn::parse(input).map(|i| i.sig))?;

        Ok((&sig).into())
    }
}

pub(crate) fn get_type(arg: &FnArg) -> &PatType {
    match arg {
        FnArg::Receiver(r) => abort!(r.span(), "instance methods not supported"),
        FnArg::Typed(pt) => pt,
    }
}

pub(crate) fn is_ret_type_complex(output: &ReturnType) -> bool {
    match output {
        ReturnType::Default => false,
        ReturnType::Type(_, ty) => is_type_complex(ty.as_ref()),
    }
}
pub(crate) fn is_type_complex(ty: &Type) -> bool {
    match ty {
        Type::Path(tp) if tp.qself.is_none() => {
            let name = tp.path.to_token_stream().to_string();
            match name.as_str() {
                "bool" | "f32" | "f64" | "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32"
                | "u64" | "usize" => false,
                _ => true,
            }
        }
        //the tuple is complex if any elements are complex
        Type::Tuple(tup) => tup
            .elems
            .iter()
            .map(is_type_complex)
            .collect::<Vec<_>>()
            .into_iter()
            .any(std::convert::identity),
        t => abort!(t, "unsupported type"),
    }
}

pub(crate) fn type_check_export(map: &HashMap<String, FnSignature>, sig: &Signature) {
    let checked: FnSignature = match sig.try_into() {
        Ok(sig) => sig,
        Err(e) => abort!(sig, "failed to convert signature: {:?}", e),
    };

    let exported = match map.get(&checked.name) {
        Some(e) => e,
        None => {
            println!("SPAN: {:?}", sig.ident.span());
            abort!(
                sig.ident,
                "no exported function named `{}` found",
                checked.name
            )
        }
    };

    if exported.is_async != checked.is_async {
        emit_error!(
            sig.asyncness,
            "the exported function `{}` should{} be async",
            checked.name,
            if !exported.is_async { " not" } else { "" }
        );
    }

    if exported.is_const != checked.is_const {
        emit_error!(
            sig.constness,
            "the exported function `{}` should{} be const",
            checked.name,
            if !exported.is_const { " not" } else { "" }
        );
    }

    if exported.output != checked.output {
        emit_error!(
            sig,
            "mismatched types expected `{}` got `{}`",
            exported.output,
            checked.output
        );
    }

    let inputs = sig.inputs.iter().collect::<Vec<_>>();

    let num_inputs_expected = exported.inputs.len();
    let num_inputs_actual = checked.inputs.len();
    let max_inputs = max(num_inputs_actual, num_inputs_expected);

    for i in 0..max_inputs {
        match (exported.inputs.get(i), checked.inputs.get(i)) {
            (Some(expected), Some(actual)) => {
                if expected.ty != actual.ty {
                    emit_error!(
                        inputs[i].span(),
                        "mismatched argument type got `{}` but expected `{}`",
                        actual.ty,
                        expected.ty
                    );
                }
            }
            (Some(expected), None) => {
                emit_error!(
                    sig.inputs,
                    "missing argument `{}: {}`",
                    expected.name,
                    expected.ty
                );
            }
            (None, Some(actual)) => emit_error!(
                inputs[i].span(),
                "invalid argument `{}`, not part of exported signature",
                actual.name,
            ),
            _ => unreachable!(),
        }
    }
}
