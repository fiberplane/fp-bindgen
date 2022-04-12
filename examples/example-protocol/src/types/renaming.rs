use fp_bindgen::prelude::Serializable;
use serde::{Deserialize, Serialize};

// Properties and variants can be renamed through the same annotations as those
// supported by Serde.
//
// If a type derives Serde's `Serialize` or `Deserialize` trait, the Serde
// annotations that you provide through `#[serde(...)]` will be picked up
// automatically by fp-bindgen's `Serializable` derive macro. This way, both
// Serde and fp-bindgen will use the same representation during serialization.
//
// Otherwise, you can provide the same annotations inside `#[fp(...)]`.
//
// For more information, see the supported annotations:
// - https://serde.rs/container-attrs.html#rename_all
// - https://serde.rs/field-attrs.html#rename
// - https://serde.rs/variant-attrs.html#rename
// - https://serde.rs/variant-attrs.html#rename_all

// This struct renames its properties using the `fp` attribute namespace.
#[derive(Serializable)]
#[fp(rename_all = "camelCase")]
pub struct FpPropertyRenaming {
    // Will be renamed to "fooBar" because of the `rename_all` on the struct.
    pub foo_bar: String,

    // Custom property name.
    #[fp(rename = "QUX_BAZ")]
    pub qux_baz: f64,

    // Raw identifiers are supported and will be processed like any other.
    pub r#raw_struct: i32,
}

// This enum renames its variants using the `fp` attribute namespace.
#[derive(Serializable)]
#[fp(rename_all = "snake_case")]
pub enum FpVariantRenaming {
    // Will be renamed to "foo_bar" because of the `rename_all` on the enum.
    FooBar,
    // Custom variant name, while its properties will be renamed also.
    #[fp(rename = "QUX_BAZ", rename_all = "SCREAMING_SNAKE_CASE")]
    QuxBaz {
        /// Will be renamed to "FOO_BAR" because of the `rename_all` on the
        /// variant.
        foo_bar: String,

        // Custom property name.
        #[fp(rename = "qux_baz")]
        qux_baz: f64,
    },
}

// This struct renames its properties using the `serde` attribute namespace.
#[derive(Serializable, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerdePropertyRenaming {
    // Will be renamed to "fooBar" because of the `rename_all` on the struct.
    pub foo_bar: String,

    // Custom property name.
    #[serde(rename = "QUX_BAZ")]
    pub qux_baz: f64,

    // Raw identifiers are supported and will be processed like any other.
    pub r#raw_struct: i32,
}

// This enum renames its variants using the `serde` attribute namespace.
#[derive(Serializable, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SerdeVariantRenaming {
    // Will be renamed to "foo_bar" because of the `rename_all` on the enum.
    FooBar,
    // Custom variant name, while its properties will be renamed also.
    #[serde(rename = "QUX_BAZ", rename_all = "PascalCase")]
    QuxBaz {
        /// Will be renamed to "FooBar" because of the `rename_all` on the
        /// variant.
        foo_bar: String,

        // Custom property name.
        #[serde(rename = "qux_baz")]
        qux_baz: f64,
    },
}
