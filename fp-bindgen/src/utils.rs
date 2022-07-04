use syn::{ReturnType, Type};

// This is the same as the one in macros/src/lib.rs, which we unfortunately cannot export.
pub(crate) fn normalize_return_type(ty: &ReturnType) -> Option<&Type> {
    match ty {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => {
            match ty.as_ref() {
                Type::Tuple(tuple) if tuple.elems.is_empty() => {
                    /* An empty '-> ()' return value */
                    None
                }
                r => Some(r),
            }
        }
    }
}
