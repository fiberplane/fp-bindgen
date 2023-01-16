use crate::types::*;

#[fp_bindgen_support::fp_export_signature]
pub fn export_array_f32(arg: [f32; 3]) -> [f32; 3];

#[fp_bindgen_support::fp_export_signature]
pub fn export_array_f64(arg: [f64; 3]) -> [f64; 3];

#[fp_bindgen_support::fp_export_signature]
pub fn export_array_i16(arg: [i16; 3]) -> [i16; 3];

#[fp_bindgen_support::fp_export_signature]
pub fn export_array_i32(arg: [i32; 3]) -> [i32; 3];

#[fp_bindgen_support::fp_export_signature]
pub fn export_array_i8(arg: [i8; 3]) -> [i8; 3];

#[fp_bindgen_support::fp_export_signature]
pub fn export_array_u16(arg: [u16; 3]) -> [u16; 3];

#[fp_bindgen_support::fp_export_signature]
pub fn export_array_u32(arg: [u32; 3]) -> [u32; 3];

#[fp_bindgen_support::fp_export_signature]
pub fn export_array_u8(arg: [u8; 3]) -> [u8; 3];

#[fp_bindgen_support::fp_export_signature]
pub async fn export_async_struct(arg1: FpPropertyRenaming, arg2: u64) -> FpPropertyRenaming;

#[fp_bindgen_support::fp_export_signature]
pub fn export_fp_adjacently_tagged(arg: FpAdjacentlyTagged) -> FpAdjacentlyTagged;

#[fp_bindgen_support::fp_export_signature]
pub fn export_fp_enum(arg: FpVariantRenaming) -> FpVariantRenaming;

#[fp_bindgen_support::fp_export_signature]
pub fn export_fp_flatten(arg: FpFlatten) -> FpFlatten;

#[fp_bindgen_support::fp_export_signature]
pub fn export_fp_internally_tagged(arg: FpInternallyTagged) -> FpInternallyTagged;

#[fp_bindgen_support::fp_export_signature]
pub fn export_fp_struct(arg: FpPropertyRenaming) -> FpPropertyRenaming;

#[fp_bindgen_support::fp_export_signature]
pub fn export_fp_untagged(arg: FpUntagged) -> FpUntagged;

#[fp_bindgen_support::fp_export_signature]
pub fn export_generics(arg: StructWithGenerics<u64>) -> StructWithGenerics<u64>;

#[fp_bindgen_support::fp_export_signature]
pub fn export_get_bytes() -> Result<bytes::Bytes, String>;

#[fp_bindgen_support::fp_export_signature]
pub fn export_get_serde_bytes() -> Result<serde_bytes::ByteBuf, String>;

#[fp_bindgen_support::fp_export_signature]
pub fn export_multiple_primitives(arg1: i8, arg2: String) -> i64;

#[fp_bindgen_support::fp_export_signature]
pub fn export_primitive_bool(arg: bool) -> bool;

#[fp_bindgen_support::fp_export_signature]
pub fn export_primitive_f32(arg: f32) -> f32;

#[fp_bindgen_support::fp_export_signature]
pub fn export_primitive_f64(arg: f64) -> f64;

#[fp_bindgen_support::fp_export_signature]
pub fn export_primitive_i16(arg: i16) -> i16;

#[fp_bindgen_support::fp_export_signature]
pub fn export_primitive_i32(arg: i32) -> i32;

#[fp_bindgen_support::fp_export_signature]
pub fn export_primitive_i64(arg: i64) -> i64;

#[fp_bindgen_support::fp_export_signature]
pub fn export_primitive_i8(arg: i8) -> i8;

#[fp_bindgen_support::fp_export_signature]
pub fn export_primitive_u16(arg: u16) -> u16;

#[fp_bindgen_support::fp_export_signature]
pub fn export_primitive_u32(arg: u32) -> u32;

#[fp_bindgen_support::fp_export_signature]
pub fn export_primitive_u64(arg: u64) -> u64;

#[fp_bindgen_support::fp_export_signature]
pub fn export_primitive_u8(arg: u8) -> u8;

#[fp_bindgen_support::fp_export_signature]
pub fn export_serde_adjacently_tagged(arg: SerdeAdjacentlyTagged) -> SerdeAdjacentlyTagged;

#[fp_bindgen_support::fp_export_signature]
pub fn export_serde_enum(arg: SerdeVariantRenaming) -> SerdeVariantRenaming;

#[fp_bindgen_support::fp_export_signature]
pub fn export_serde_flatten(arg: SerdeFlatten) -> SerdeFlatten;

#[fp_bindgen_support::fp_export_signature]
pub fn export_serde_internally_tagged(arg: SerdeInternallyTagged) -> SerdeInternallyTagged;

#[fp_bindgen_support::fp_export_signature]
pub fn export_serde_struct(arg: SerdePropertyRenaming) -> SerdePropertyRenaming;

#[fp_bindgen_support::fp_export_signature]
pub fn export_serde_untagged(arg: SerdeUntagged) -> SerdeUntagged;

#[fp_bindgen_support::fp_export_signature]
pub fn export_string(arg: String) -> String;

#[fp_bindgen_support::fp_export_signature]
pub fn export_struct_with_options(arg: StructWithOptions) -> StructWithOptions;

#[fp_bindgen_support::fp_export_signature]
pub fn export_timestamp(arg: MyDateTime) -> MyDateTime;

#[fp_bindgen_support::fp_export_signature]
pub fn export_void_function();

/// Example how plugin could expose async data-fetching capabilities.
#[fp_bindgen_support::fp_export_signature]
pub async fn fetch_data(r#type: String) -> Result<String, String>;

/// Called on the plugin to give it a chance to initialize.
#[fp_bindgen_support::fp_export_signature]
pub fn init();

/// Example how plugin could expose a reducer.
#[fp_bindgen_support::fp_export_signature]
pub fn reducer_bridge(action: ReduxAction) -> StateUpdate;
