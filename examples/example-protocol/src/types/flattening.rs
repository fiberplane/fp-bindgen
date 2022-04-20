use fp_bindgen::prelude::Serializable;
use serde::{Deserialize, Serialize};

// Structs can be flattened using the `#[fp(flatten)]` annotation.
//
// If a type derives Serde's `Serialize` or `Deserialize` trait, the Serde
// annotations `#[serde(flatten)]` can be used instead. This way, both
// Serde and fp-bindgen will use the same representation during serialization.
//
// For more information, see: https://serde.rs/attr-flatten.html

#[derive(Serializable)]
pub struct FpFlatten {
    // The `flattened` field will not be part of the serialized representation.
    // Instead, `foo` and `bar` will be serialized directly.
    #[fp(flatten)]
    pub flattened: FlattenedStruct,
}

#[derive(Serializable, Serialize, Deserialize)]
pub struct SerdeFlatten {
    // The `flattened` field will not be part of the serialized representation.
    // Instead, `foo` and `bar` will be serialized directly.
    #[serde(flatten)]
    pub flattened: FlattenedStruct,
}

// This struct will not be serialized as-is, but its properties will become
// part of the structs that include it.
#[derive(Serializable, Serialize, Deserialize)]
pub struct FlattenedStruct {
    pub foo: String,
    pub bar: i64,
}
