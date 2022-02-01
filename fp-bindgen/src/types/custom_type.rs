use super::{CargoDependency, TypeIdent};
use std::{collections::BTreeMap, hash::Hash};

/// Used for defining type information for types that are defined externally,
/// or that otherwise require custom treatment.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CustomType {
    pub ident: TypeIdent,

    /// Qualified path to refer to the type in the Rust generators.
    pub rs_ty: String,

    /// Dependencies to add to the Rust plugin's `Cargo.toml` to be able to
    /// use the type.
    ///
    /// Keys in the map are dependency names as they appear on the left-hand
    /// side of the `=` in the `Cargo.toml` `[dependencies]` section, while the
    /// value describes what comes on the right-hand side.
    pub rs_dependencies: BTreeMap<&'static str, CargoDependency>,

    /// Serde attributes to add to fields of this type.
    pub serde_attrs: Vec<String>,

    /// Name to refer to the type in the TypeScript generator.
    pub ts_ty: String,

    /// Optional declaration, for when `ts_ty` does not refer to a built-in
    /// type.
    pub ts_declaration: Option<String>,
}
