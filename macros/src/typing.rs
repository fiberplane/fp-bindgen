use proc_macro_error::{abort, ResultExt};
use quote::ToTokens;
use syn::{spanned::Spanned, FnArg, PatType, ReturnType, Signature, Type};

pub(crate) fn get_pat_type(arg: &FnArg) -> &PatType {
    match arg {
        FnArg::Receiver(r) => abort!(r.span(), "instance methods not supported"),
        FnArg::Typed(pt) => pt,
    }
}
pub(crate) fn get_pat_type_mut(arg: &mut FnArg) -> &mut PatType {
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

pub(crate) fn get_output_type(output: &ReturnType) -> &Type {
    match output {
        ReturnType::Default => abort!(output, "FIXME"),
        ReturnType::Type(_, ty) => ty.as_ref(),
    }
}

pub(crate) fn replace_complex_type(ty: &mut Type) {
    if is_type_complex(ty) {
        *ty = syn::parse_str::<Type>("fp_bindgen_lib::FatPtr").unwrap_or_abort();
    }
}

/// Replaces complex types in the input and output of a function signature and makes it non-async
pub(crate) fn morph_signature(sig: &mut Signature) {
    sig.asyncness = None;
    sig.inputs = sig
        .inputs
        .iter()
        .cloned()
        .map(|mut arg| {
            let pt = get_pat_type_mut(&mut arg);
            replace_complex_type(&mut pt.ty);
            arg
        })
        .collect();

    if let ReturnType::Type(_, ref mut ty) = sig.output {
        replace_complex_type(ty.as_mut());
    }
}

pub(crate) fn extract_args(sig: &Signature) -> impl Iterator<Item = (&FnArg, &PatType, bool)> {
    sig.inputs.iter().map(|arg| {
        let pt = get_pat_type(arg);
        (arg, pt, is_type_complex(&pt.ty))
    })
}
