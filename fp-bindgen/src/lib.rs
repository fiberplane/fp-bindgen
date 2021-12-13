mod casing;
mod docs;
mod functions;
#[cfg(feature = "generators")]
mod generators;
mod serializable;

pub mod generics;
pub mod prelude;
pub mod primitives;
pub mod types;

use fp_bindgen_macros::primitive_impls;
use prelude::*;
use std::collections::BTreeSet;

primitive_impls!();

#[cfg(feature = "generators")]
pub use generators::{
    generate_bindings, BindingConfig, BindingsType, RustPluginConfig, TsRuntimeConfig,
};
