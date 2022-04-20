use fp_bindgen::prelude::Serializable;

// Rust doc comments are supported on protocol functions, structs, enums,
// variants and properties.

// Generic arguments can be used, both on `std` types that take generic
// arguments as well as custom types with a `Serializable` implementation.

/// # This is a struct with doc comments.
#[derive(Serializable)]
pub struct DocExampleStruct {
    /// Multi-line doc comment with complex characters
    /// & " , \ ! '
    pub multi_line: String,

    /// Raw identifiers are supported too.
    pub r#type: String,
}

/// # This is an enum with doc comments.
#[derive(Serializable)]
pub enum DocExampleEnum {
    /// Multi-line doc comment with complex characters
    /// & " , \ ! '
    Variant1(String),

    /// Raw identifiers are supported too.
    r#Variant2 {
        /// Variant property.
        inner: i8,
    },
}
