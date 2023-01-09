use fp_bindgen::prelude::Serializable;
use time::OffsetDateTime;

// This example shows how types from the `time` crate can be communicated.
//
// For these types it is important that you serialize them using a string
// representation if you want to be able to use them from TypeScript. RFC3339
// is recommended because you can pass these directly to the JavaScript `Date`
// constructor. The Serde attributes to enable RFC3339 formatting are inserted
// automatically by the bindings generator, but you may have to add them
// yourself if you using annotations such as `#[fp(rust_module)]`.
//
// If you do not use RFC3339 formatting, you should expect your date/time types
// to only work from Rust to Rust.

/// Our struct for passing date time instances.
///
/// We wrap the `OffsetDateTime` type in a new struct so that the Serde
/// attributes can be inserted. These are necessary to enable RFC3339
/// formatting. Without a wrapper type like this, we would not be able to pass
/// date time instances directly to function arguments and we might run into
/// trouble embedding them into certain generic types.
#[derive(Serializable)]
pub struct MyDateTime(OffsetDateTime);
