use ::http::{Method, Uri};
use example_bindings::*;
use serde_bytes::ByteBuf;
use std::collections::{BTreeMap, HashMap};
use std::panic;
use time::{macros::datetime, OffsetDateTime};

// This plugin contains implementations for all the functions it may export
// according to the protocol. These functions are called during our integration
// tests, so they verify their arguments are as expected from the test, and
// return a value that is in turn verified by the caller in the test suite.

// The reducer module contains an example of how to a Redux reducer in Rust.
// For context, please read:
//                https://fiberplane.dev/blog/writing-redux-reducers-in-rust/
mod reducer;

fn init_panic_hook() {
    use std::sync::Once;
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        panic::set_hook(Box::new(|info| log(info.to_string())));
    });
}

#[fp_export_impl(example_bindings)]
fn export_primitive_bool(arg: bool) -> bool {
    arg
}

#[fp_export_impl(example_bindings)]
fn export_primitive_f32(arg: f32) -> f32 {
    assert!(arg > 3.14);
    assert!(arg < 3.15);
    3.1415926535
}

#[fp_export_impl(example_bindings)]
fn export_primitive_f64(arg: f64) -> f64 {
    assert!(arg > 2.7182);
    assert!(arg < 2.7183);
    2.718281828459
}

#[fp_export_impl(example_bindings)]
fn export_primitive_i8(arg: i8) -> i8 {
    assert_eq!(arg, -8);
    -8
}

#[fp_export_impl(example_bindings)]
fn export_primitive_i16(arg: i16) -> i16 {
    assert_eq!(arg, -16);
    -16
}

#[fp_export_impl(example_bindings)]
fn export_primitive_i32(arg: i32) -> i32 {
    assert_eq!(arg, -32);
    -32
}

#[fp_export_impl(example_bindings)]
fn export_primitive_i64(arg: i64) -> i64 {
    assert_eq!(arg, -64);
    -64
}

#[fp_export_impl(example_bindings)]
fn export_primitive_u8(arg: u8) -> u8 {
    assert_eq!(arg, 8);
    8
}

#[fp_export_impl(example_bindings)]
fn export_primitive_u16(arg: u16) -> u16 {
    assert_eq!(arg, 16);
    16
}

#[fp_export_impl(example_bindings)]
fn export_primitive_u32(arg: u32) -> u32 {
    assert_eq!(arg, 32);
    32
}

#[fp_export_impl(example_bindings)]
fn export_primitive_u64(arg: u64) -> u64 {
    assert_eq!(arg, 64);
    64
}

#[fp_export_impl(example_bindings)]
fn export_string(arg: String) -> String {
    assert_eq!(arg, "Hello, plugin!");
    "Hello, world!".to_owned()
}

#[fp_export_impl(example_bindings)]
fn export_multiple_primitives(arg1: i8, arg2: String) -> i64 {
    assert_eq!(arg1, -8);
    assert_eq!(arg2, "Hello, 🇳🇱!");
    -64
}

#[fp_export_impl(example_bindings)]
fn export_timestamp(arg: OffsetDateTime) -> OffsetDateTime {
    assert_eq!(arg, datetime!(2022-04-12 19:10 UTC));
    datetime!(2022-04-13 12:37 UTC)
}

#[fp_export_impl(example_bindings)]
fn export_fp_flatten(arg: FpFlatten) -> FpFlatten {
    assert_eq!(
        arg,
        FpFlatten {
            flattened: FlattenedStruct {
                foo: "Hello, 🇳🇱!".to_owned(),
                bar: -64,
            }
        }
    );
    FpFlatten {
        flattened: FlattenedStruct {
            foo: "Hello, 🇩🇪!".to_owned(),
            bar: -64,
        },
    }
}

#[fp_export_impl(example_bindings)]
fn export_serde_flatten(arg: SerdeFlatten) -> SerdeFlatten {
    assert_eq!(
        arg,
        SerdeFlatten {
            flattened: FlattenedStruct {
                foo: "Hello, 🇳🇱!".to_owned(),
                bar: -64,
            }
        }
    );
    SerdeFlatten {
        flattened: FlattenedStruct {
            foo: "Hello, 🇩🇪!".to_owned(),
            bar: -64,
        },
    }
}

#[fp_export_impl(example_bindings)]
fn export_generics(arg: StructWithGenerics<u64>) -> StructWithGenerics<u64> {
    assert_eq!(
        arg,
        StructWithGenerics {
            list: vec![0, 64],
            points: vec![Point { value: 64 }],
            recursive: vec![Point {
                value: Point { value: 64 }
            }],
            complex_nested: Some(BTreeMap::from([
                ("one".to_owned(), vec![Point { value: 1.0 }]),
                ("two".to_owned(), vec![Point { value: 2.0 }])
            ])),
            optional_timestamp: Some(OffsetDateTime::UNIX_EPOCH)
        }
    );
    StructWithGenerics {
        list: vec![0, 64],
        points: vec![Point { value: 64 }],
        recursive: vec![Point {
            value: Point { value: 64 },
        }],
        complex_nested: Some(BTreeMap::from([
            ("een".to_owned(), vec![Point { value: 1.0 }]),
            ("twee".to_owned(), vec![Point { value: 2.0 }]),
        ])),
        optional_timestamp: Some(OffsetDateTime::UNIX_EPOCH),
    }
}

#[fp_export_impl(example_bindings)]
fn export_fp_struct(arg: FpPropertyRenaming) -> FpPropertyRenaming {
    assert_eq!(
        arg,
        FpPropertyRenaming {
            foo_bar: "foo_bar".to_owned(),
            qux_baz: 64.0,
            raw_struct: -32
        }
    );
    FpPropertyRenaming {
        foo_bar: "fooBar".to_owned(),
        qux_baz: -64.0,
        raw_struct: 32,
    }
}

#[fp_export_impl(example_bindings)]
fn export_fp_enum(arg: FpVariantRenaming) -> FpVariantRenaming {
    assert_eq!(arg, FpVariantRenaming::FooBar);
    FpVariantRenaming::QuxBaz {
        foo_bar: "foo_bar".to_owned(),
        qux_baz: 64.0,
    }
}

#[fp_export_impl(example_bindings)]
fn export_serde_struct(arg: SerdePropertyRenaming) -> SerdePropertyRenaming {
    assert_eq!(
        arg,
        SerdePropertyRenaming {
            foo_bar: "foo_bar".to_owned(),
            qux_baz: 64.0,
            raw_struct: -32
        }
    );
    SerdePropertyRenaming {
        foo_bar: "fooBar".to_owned(),
        qux_baz: -64.0,
        raw_struct: 32,
    }
}

#[fp_export_impl(example_bindings)]
fn export_serde_enum(arg: SerdeVariantRenaming) -> SerdeVariantRenaming {
    assert_eq!(arg, SerdeVariantRenaming::FooBar);
    SerdeVariantRenaming::QuxBaz {
        foo_bar: "foo_bar".to_owned(),
        qux_baz: 64.0,
    }
}

#[fp_export_impl(example_bindings)]
fn export_fp_internally_tagged(arg: FpInternallyTagged) -> FpInternallyTagged {
    assert_eq!(arg, FpInternallyTagged::Foo);
    FpInternallyTagged::Baz { a: -8, b: 64 }
}

#[fp_export_impl(example_bindings)]
fn export_fp_adjacently_tagged(arg: FpAdjacentlyTagged) -> FpAdjacentlyTagged {
    assert_eq!(arg, FpAdjacentlyTagged::Bar("Hello, plugin!".to_owned()));
    FpAdjacentlyTagged::Baz { a: -8, b: 64 }
}

#[fp_export_impl(example_bindings)]
fn export_fp_untagged(arg: FpUntagged) -> FpUntagged {
    assert_eq!(arg, FpUntagged::Bar("Hello, plugin!".to_owned()));
    FpUntagged::Baz { a: -8, b: 64 }
}

#[fp_export_impl(example_bindings)]
fn export_serde_internally_tagged(arg: SerdeInternallyTagged) -> SerdeInternallyTagged {
    assert_eq!(arg, SerdeInternallyTagged::Foo);
    SerdeInternallyTagged::Baz { a: -8, b: 64 }
}

#[fp_export_impl(example_bindings)]
fn export_serde_adjacently_tagged(arg: SerdeAdjacentlyTagged) -> SerdeAdjacentlyTagged {
    assert_eq!(arg, SerdeAdjacentlyTagged::Bar("Hello, plugin!".to_owned()));
    SerdeAdjacentlyTagged::Baz { a: -8, b: 64 }
}

#[fp_export_impl(example_bindings)]
fn export_serde_untagged(arg: SerdeUntagged) -> SerdeUntagged {
    assert_eq!(arg, SerdeUntagged::Bar("Hello, plugin!".to_owned()));
    SerdeUntagged::Baz { a: -8, b: 64 }
}

#[fp_export_impl(example_bindings)]
async fn export_async_struct(arg1: FpPropertyRenaming, arg2: u64) -> FpPropertyRenaming {
    assert_eq!(
        arg1,
        FpPropertyRenaming {
            foo_bar: "foo_bar".to_owned(),
            qux_baz: 64.0,
            raw_struct: -32
        }
    );
    assert_eq!(arg2, 64);
    FpPropertyRenaming {
        foo_bar: "fooBar".to_owned(),
        qux_baz: -64.0,
        raw_struct: 32,
    }
}

#[fp_export_impl(example_bindings)]
async fn fetch_data(r#type: String) -> Result<String, String> {
    let result = make_http_request(Request {
        url: Uri::from_static("https://fiberplane.dev"),
        method: Method::POST,
        headers: HashMap::from([("Content-Type".to_owned(), "application/json".to_owned())]),
        body: Some(ByteBuf::from(format!(
            r#"{{"country":"🇳🇱","type":"{}"}}"#,
            r#type
        ))),
    })
    .await;

    match result {
        Ok(response) => {
            String::from_utf8(response.body.to_vec()).map_err(|_| "Invalid utf8".to_owned())
        }
        Err(err) => Err(format!("Error: {:?}", err)),
    }
}

#[fp_export_impl(example_bindings)]
fn init() {
    init_panic_hook();
}
