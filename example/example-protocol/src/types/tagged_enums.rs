use fp_bindgen::prelude::Serializable;
use serde::{Deserialize, Serialize};

// Enums are (usually) tagged during serialization so the deserializer can
// determine which variant is encoded.
//
// fp-bindgen follows Serde conventions regarding tagging. Externally tagged
// (the default), internally tagged, adjacently tagged and untagged are all
// supported.
//
// If the type in your protocol derives Serde's `Serialize` or `Deserialize`
// trait, fp-bindgen's `Serializable` derive can pick up automatically on
// `#[serde(...)]` annotations for tagging.
//
// Otherwise, you can use the same annotations using `#[fp(...)]`.
//
// For more information, see: https://serde.rs/enum-representations.html

#[derive(Serializable)]
#[fp(tag = "type")]
pub enum FpInternallyTagged {
    Foo,
    // Internally tagged enums cannot have unnamed fields!
    //Bar(String), // NOT SUPPORTED!
    Baz { a: i8, b: u64 },
}

#[derive(Serializable)]
#[fp(tag = "type", content = "payload")]
pub enum FpAdjacentlyTagged {
    Foo,
    Bar(String),
    Baz { a: i8, b: u64 },
}

#[derive(Serializable)]
#[fp(untagged)]
pub enum FpUntagged {
    // Untagged enums must have inner fields!
    //Foo, // NOT SUPPORTED!
    Bar(String),
    Baz { a: i8, b: u64 },
}

#[derive(Serializable, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SerdeInternallyTagged {
    Foo,
    // Internally tagged enums cannot have unnamed fields!
    //Bar(String), // NOT SUPPORTED!
    Baz { a: i8, b: u64 },
}

#[derive(Serializable, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum SerdeAdjacentlyTagged {
    Foo,
    Bar(String),
    Baz { a: i8, b: u64 },
}

#[derive(Serializable, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SerdeUntagged {
    // Untagged enums must have inner fields!
    //Foo, // NOT SUPPORTED!
    Bar(String),
    Baz { a: i8, b: u64 },
}
