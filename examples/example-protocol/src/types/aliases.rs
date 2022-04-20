use serde_bytes::ByteBuf;

use super::{RequestError, Response};

// Aliases are supported, but in order for them to appear correctly in the
// generated bindings, they need to be repeated in either the `fp_import!` or
// the `fp_export!` bindings.
//
// This is unfortunately necessary because aliases cannot be annotated :(

pub type Body = ByteBuf;

pub type HttpResult = Result<Response, RequestError>;

// This alias is annoying for the TypeScript bindings, because 64-bit numbers
// could be encoded using numbers or the `BigInt` type depending on context.
// An alias doesn't know its context, so it needs to account for either case.
pub type Int64 = u64;
