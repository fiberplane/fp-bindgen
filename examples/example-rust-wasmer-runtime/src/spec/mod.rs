pub mod bindings;
pub mod types;

use bytes::Bytes;
use serde_bytes::ByteBuf;
use types::*;

fn import_void_function() {}
fn import_void_function_empty_result() -> Result<(), u32> {
    Ok(())
}
fn import_void_function_empty_return() -> () {}

fn import_explicit_bound_point(arg: ExplicitBoundPoint<u64>) {
    todo!()
}
fn import_primitive_bool(arg: bool) -> bool {
    todo!()
}
fn import_primitive_f32(arg: f32) -> f32 {
    todo!()
}
fn import_primitive_f64(arg: f64) -> f64 {
    todo!()
}
fn import_primitive_i8(arg: i8) -> i8 {
    todo!()
}
fn import_primitive_i16(arg: i16) -> i16 {
    todo!()
}
fn import_primitive_i32(arg: i32) -> i32 {
    todo!()
}
fn import_primitive_i64(arg: i64) -> i64 {
    todo!()
}
fn import_primitive_u8(arg: u8) -> u8 {
    todo!()
}
fn import_primitive_u16(arg: u16) -> u16 {
    todo!()
}
fn import_primitive_u32(arg: u32) -> u32 {
    todo!()
}
fn import_primitive_u64(arg: u64) -> u64 {
    todo!()
}

fn import_array_u8(arg: [u8; 3]) -> [u8; 3] {
    todo!()
}
fn import_array_u16(arg: [u16; 3]) -> [u16; 3] {
    todo!()
}
fn import_array_u32(arg: [u32; 3]) -> [u32; 3] {
    todo!()
}
fn import_array_i8(arg: [i8; 3]) -> [i8; 3] {
    todo!()
}
fn import_array_i16(arg: [i16; 3]) -> [i16; 3] {
    todo!()
}
fn import_array_i32(arg: [i32; 3]) -> [i32; 3] {
    todo!()
}
fn import_array_f32(arg: [f32; 3]) -> [f32; 3] {
    todo!()
}
fn import_array_f64(arg: [f64; 3]) -> [f64; 3] {
    todo!()
}

fn import_string(arg: String) -> String {
    todo!()
}

fn import_multiple_primitives(arg1: i8, arg2: String) -> i64 {
    todo!()
}

fn import_timestamp(arg: MyDateTime) -> MyDateTime {
    todo!()
}

fn import_fp_flatten(arg: FpFlatten) -> FpFlatten {
    todo!()
}
fn import_serde_flatten(arg: SerdeFlatten) -> SerdeFlatten {
    todo!()
}

fn import_generics(arg: StructWithGenerics<u64>) -> StructWithGenerics<u64> {
    todo!()
}

fn import_get_bytes() -> Result<Bytes, String> {
    Ok(Bytes::from("hello"))
}
fn import_get_serde_bytes() -> Result<ByteBuf, String> {
    Ok(ByteBuf::from("hello"))
}

fn import_fp_struct(arg: FpPropertyRenaming) -> FpPropertyRenaming {
    todo!()
}
fn import_fp_enum(arg: FpVariantRenaming) -> FpVariantRenaming {
    todo!()
}
fn import_serde_struct(arg: SerdePropertyRenaming) -> SerdePropertyRenaming {
    todo!()
}
fn import_serde_enum(arg: SerdeVariantRenaming) -> SerdeVariantRenaming {
    todo!()
}

fn import_fp_internally_tagged(arg: FpInternallyTagged) -> FpInternallyTagged {
    todo!()
}
fn import_fp_adjacently_tagged(arg: FpAdjacentlyTagged) -> FpAdjacentlyTagged {
    todo!()
}
fn import_fp_untagged(arg: FpUntagged) -> FpUntagged {
    todo!()
}
fn import_serde_internally_tagged(arg: SerdeInternallyTagged) -> SerdeInternallyTagged {
    todo!()
}
fn import_serde_adjacently_tagged(arg: SerdeAdjacentlyTagged) -> SerdeAdjacentlyTagged {
    todo!()
}
fn import_serde_untagged(arg: SerdeUntagged) -> SerdeUntagged {
    todo!()
}

fn import_struct_with_options(arg: StructWithOptions) {
    todo!()
}

fn log(msg: String) {
    println!("Provider log: {}", msg);
}

async fn make_http_request(opts: Request) -> Result<Response, RequestError> {
    Ok(Response {
        body: ByteBuf::from(r#"status: "confirmed"#.to_string()),
        headers: opts.headers,
        status_code: 200,
    })
}
