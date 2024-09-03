pub mod bindings;
pub mod types;

use bytes::Bytes;
use serde_bytes::ByteBuf;
use tokio::time::{sleep, Duration};
use types::*;
use super::GLOBAL_STATE;

fn import_void_function() {}
fn import_void_function_empty_result() -> Result<(), u32> {
    Ok(())
}
fn import_void_function_empty_return() -> () {}

fn import_explicit_bound_point(_arg: ExplicitBoundPoint<u64>) {
    todo!()
}
fn import_primitive_bool_negate(arg: bool) -> bool {
    !arg
}
fn import_primitive_f32_add_one(arg: f32) -> f32 {
    arg + 1.0
}
fn import_primitive_f64_add_one(arg: f64) -> f64 {
    arg + 1.0
}
fn import_primitive_f32_add_one_wasmer2(arg: [f32; 1]) -> f32 {
    arg[0] + 1.0
}
fn import_primitive_f64_add_one_wasmer2(arg: [f64; 1]) -> f64 {
    arg[0] + 1.0
}
fn import_primitive_i8_add_one(arg: i8) -> i8 {
    arg + 1
}
fn import_primitive_i16_add_one(arg: i16) -> i16 {
    arg + 1
}
fn import_primitive_i32_add_one(arg: i32) -> i32 {
    arg + 1
}
fn import_primitive_i64_add_one(arg: i64) -> i64 {
    arg + 1
}
fn import_primitive_u8_add_one(arg: u8) -> u8 {
    arg + 1
}
fn import_primitive_u16_add_one(arg: u16) -> u16 {
    arg + 1
}
fn import_primitive_u32_add_one(arg: u32) -> u32 {
    arg + 1
}
fn import_primitive_u64_add_one(arg: u64) -> u64 {
    arg + 1
}

fn import_array_u8(_arg: [u8; 3]) -> [u8; 3] {
    todo!()
}
fn import_array_u16(_arg: [u16; 3]) -> [u16; 3] {
    todo!()
}
fn import_array_u32(_arg: [u32; 3]) -> [u32; 3] {
    todo!()
}
fn import_array_i8(_arg: [i8; 3]) -> [i8; 3] {
    todo!()
}
fn import_array_i16(_arg: [i16; 3]) -> [i16; 3] {
    todo!()
}
fn import_array_i32(_arg: [i32; 3]) -> [i32; 3] {
    todo!()
}
fn import_array_f32(_arg: [f32; 3]) -> [f32; 3] {
    todo!()
}
fn import_array_f64(_arg: [f64; 3]) -> [f64; 3] {
    todo!()
}

fn import_string(_arg: String) -> String {
    todo!()
}

fn import_multiple_primitives(_arg1: i8, _arg2: String) -> i64 {
    todo!()
}

fn import_timestamp(_arg: MyDateTime) -> MyDateTime {
    todo!()
}

fn import_fp_flatten(_arg: FpFlatten) -> FpFlatten {
    todo!()
}
fn import_serde_flatten(_arg: SerdeFlatten) -> SerdeFlatten {
    todo!()
}

fn import_generics(_arg: StructWithGenerics<u64>) -> StructWithGenerics<u64> {
    todo!()
}

fn import_get_bytes() -> Result<Bytes, String> {
    Ok(Bytes::from("hello"))
}
fn import_get_serde_bytes() -> Result<ByteBuf, String> {
    Ok(ByteBuf::from("hello"))
}

fn import_fp_struct(_arg: FpPropertyRenaming) -> FpPropertyRenaming {
    todo!()
}
fn import_fp_enum(_arg: FpVariantRenaming) -> FpVariantRenaming {
    todo!()
}
fn import_serde_struct(_arg: SerdePropertyRenaming) -> SerdePropertyRenaming {
    todo!()
}
fn import_serde_enum(_arg: SerdeVariantRenaming) -> SerdeVariantRenaming {
    todo!()
}

fn import_fp_internally_tagged(_arg: FpInternallyTagged) -> FpInternallyTagged {
    todo!()
}
fn import_fp_adjacently_tagged(_arg: FpAdjacentlyTagged) -> FpAdjacentlyTagged {
    todo!()
}
fn import_fp_untagged(_arg: FpUntagged) -> FpUntagged {
    todo!()
}
fn import_serde_internally_tagged(_arg: SerdeInternallyTagged) -> SerdeInternallyTagged {
    todo!()
}
fn import_serde_adjacently_tagged(_arg: SerdeAdjacentlyTagged) -> SerdeAdjacentlyTagged {
    todo!()
}
fn import_serde_untagged(_arg: SerdeUntagged) -> SerdeUntagged {
    todo!()
}

async fn import_primitive_bool_negate_async(arg: bool) -> bool {
    !arg
}
async fn import_primitive_f32_add_one_async(arg: f32) -> f32 {
    arg + 1.0
}
async fn import_primitive_f64_add_one_async(arg: f64) -> f64 {
    arg + 1.0
}
async fn import_primitive_i8_add_one_async(arg: i8) -> i8 {
    arg + 1
}
async fn import_primitive_i16_add_one_async(arg: i16) -> i16 {
    arg + 1
}
async fn import_primitive_i32_add_one_async(arg: i32) -> i32 {
    arg + 1
}
async fn import_primitive_i64_add_one_async(arg: i64) -> i64 {
    arg + 1
}
async fn import_primitive_u8_add_one_async(arg: u8) -> u8 {
    arg + 1
}
async fn import_primitive_u16_add_one_async(arg: u16) -> u16 {
    arg + 1
}
async fn import_primitive_u32_add_one_async(arg: u32) -> u32 {
    arg + 1
}
async fn import_primitive_u64_add_one_async(arg: u64) -> u64 {
    arg + 1
}

async fn import_reset_global_state() {
    //GLOBAL_STATE.set(0);
    *GLOBAL_STATE.lock().unwrap() = 0;
}
async fn import_increment_global_state() {
    // Possible "race condition", but we don't mind here
    // let val = GLOBAL_STATE.get();
    // GLOBAL_STATE.set(val + 1);
    let mut lock = GLOBAL_STATE.lock().unwrap();
    let value = *lock + 1;
    *lock = value;
}

fn import_struct_with_options(_arg: StructWithOptions) {
    todo!()
}

fn log(msg: String) {
    println!("Provider log: {}", msg);
}

async fn make_http_request(opts: Request) -> Result<Response, RequestError> {
    // Add a little randomized sleeping to see if we trigger potential race issues...
    //sleep(Duration::from_millis(rand::random::<u8>().into())).await;

    Ok(Response {
        body: ByteBuf::from(r#"{"status":"confirmed"}"#.to_string()),
        headers: opts.headers,
        status_code: 200,
    })
}
