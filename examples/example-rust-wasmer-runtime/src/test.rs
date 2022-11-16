#[cfg(not(feature="wasi"))]
use crate::spec::types::*;
#[cfg(feature="wasi")]
use crate::wasi_spec::types::*;
#[cfg(not(feature="wasi"))]
use crate::spec::bindings::Runtime;
#[cfg(feature="wasi")]
use crate::wasi_spec::bindings::Runtime;
use anyhow::Result;
use bytes::Bytes;
use serde_bytes::ByteBuf;
use std::collections::BTreeMap;
use time::{macros::datetime, OffsetDateTime};

#[cfg(not(feature="wasi"))]
const WASM_BYTES: &'static [u8] =
    include_bytes!("../../example-plugin/target/wasm32-unknown-unknown/debug/example_plugin.wasm");
#[cfg(feature="wasi")]
const WASM_BYTES: &'static [u8] =
    include_bytes!("../../example-plugin/target/wasm32-wasi/debug/example_plugin.wasm");

#[test]
fn primitives() -> Result<()> {
    let rt = Runtime::new(WASM_BYTES)?;

    assert_eq!(rt.export_primitive_bool(true)?, true);
    assert_eq!(rt.export_primitive_bool(false)?, false);

    assert_eq!(rt.export_primitive_u8(8)?, 8);
    assert_eq!(rt.export_primitive_u16(16)?, 16);
    assert_eq!(rt.export_primitive_u32(32)?, 32);
    assert_eq!(rt.export_primitive_u64(64)?, 64);
    assert_eq!(rt.export_primitive_i8(-8)?, -8);
    assert_eq!(rt.export_primitive_i16(-16)?, -16);
    assert_eq!(rt.export_primitive_i32(-32)?, -32);
    assert_eq!(rt.export_primitive_i64(-64)?, -64);

    assert_eq!(
        rt.export_multiple_primitives(-8, "Hello, 🇳🇱!".to_string())?,
        -64
    );

    assert_eq!(rt.export_primitive_f32(3.1415926535)?, 3.1415926535);
    assert_eq!(
        rt.export_primitive_f64(2.718281828459f64)?,
        2.718281828459f64
    );
    Ok(())
}

#[test]
fn arrays() -> Result<()> {
    let rt = Runtime::new(WASM_BYTES)?;

    assert_eq!(rt.export_array_u8([1u8, 2u8, 3u8])?, [1u8, 2u8, 3u8]);
    assert_eq!(rt.export_array_u16([1u16, 2u16, 3u16])?, [1u16, 2u16, 3u16]);
    assert_eq!(rt.export_array_u32([1u32, 2u32, 3u32])?, [1u32, 2u32, 3u32]);
    assert_eq!(rt.export_array_i8([1i8, 2i8, 3i8])?, [1i8, 2i8, 3i8]);
    assert_eq!(rt.export_array_i16([1i16, 2i16, 3i16])?, [1i16, 2i16, 3i16]);
    assert_eq!(rt.export_array_i32([1i32, 2i32, 3i32])?, [1i32, 2i32, 3i32]);
    assert_eq!(rt.export_array_f32([1f32, 2f32, 3f32])?, [1f32, 2f32, 3f32]);
    assert_eq!(rt.export_array_f64([1f64, 2f64, 3f64])?, [1f64, 2f64, 3f64]);
    Ok(())
}

#[test]
fn string() -> Result<()> {
    let rt = Runtime::new(WASM_BYTES)?;
    assert_eq!(
        rt.export_string("Hello, plugin!".to_string())?,
        "Hello, world!"
    );

    Ok(())
}

#[test]
fn timestamp() -> Result<()> {
    let rt = Runtime::new(WASM_BYTES)?;
    assert_eq!(
        rt.export_timestamp(MyDateTime(datetime!(2022-04-12 19:10 UTC)))?,
        MyDateTime(datetime!(2022-04-13 12:37 UTC))
    );
    Ok(())
}

#[test]
fn flattened_structs() -> Result<()> {
    let rt = Runtime::new(WASM_BYTES)?;
    assert_eq!(
        rt.export_fp_struct(FpPropertyRenaming {
            foo_bar: "foo_bar".to_string(),
            qux_baz: 64.0,
            raw_struct: -32,
        })?,
        FpPropertyRenaming {
            foo_bar: "fooBar".to_string(),
            qux_baz: -64.0,
            raw_struct: 32,
        }
    );

    assert_eq!(
        rt.export_fp_enum(FpVariantRenaming::FooBar)?,
        FpVariantRenaming::QuxBaz {
            foo_bar: "foo_bar".to_string(),
            qux_baz: 64.0
        }
    );

    assert_eq!(
        rt.export_serde_struct(SerdePropertyRenaming {
            foo_bar: "foo_bar".to_string(),
            qux_baz: 64.0,
            raw_struct: -32
        })?,
        SerdePropertyRenaming {
            foo_bar: "fooBar".to_string(),
            qux_baz: -64.0,
            raw_struct: 32,
        }
    );

    assert_eq!(
        rt.export_serde_enum(SerdeVariantRenaming::FooBar)?,
        SerdeVariantRenaming::QuxBaz {
            foo_bar: "foo_bar".to_string(),
            qux_baz: 64.0,
        },
    );

    Ok(())
}

#[test]
fn generics() -> Result<()> {
    let rt = Runtime::new(WASM_BYTES)?;

    assert_eq!(
        rt.export_generics(StructWithGenerics {
            list: vec![0, 64],
            points: vec![Point { value: 64 }],
            recursive: vec![Point {
                value: Point { value: 64 }
            }],
            complex_nested: Some(BTreeMap::from([
                ("one".to_owned(), vec![Point { value: 1.0 }]),
                ("two".to_owned(), vec![Point { value: 2.0 }])
            ])),
            optional_timestamp: Some(MyDateTime(OffsetDateTime::UNIX_EPOCH))
        })?,
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
            optional_timestamp: Some(MyDateTime(OffsetDateTime::UNIX_EPOCH)),
        }
    );
    Ok(())
}

#[test]
fn property_renaming() -> Result<()> {
    let rt = Runtime::new(WASM_BYTES)?;

    assert_eq!(
        rt.export_fp_flatten(FpFlatten {
            flattened: FlattenedStruct {
                foo: "Hello, 🇳🇱!".to_owned(),
                bar: -64,
            }
        })?,
        FpFlatten {
            flattened: FlattenedStruct {
                foo: "Hello, 🇩🇪!".to_owned(),
                bar: -64,
            },
        }
    );

    assert_eq!(
        rt.export_serde_flatten(SerdeFlatten {
            flattened: FlattenedStruct {
                foo: "Hello, 🇳🇱!".to_owned(),
                bar: -64,
            }
        })?,
        SerdeFlatten {
            flattened: FlattenedStruct {
                foo: "Hello, 🇩🇪!".to_owned(),
                bar: -64,
            },
        }
    );

    Ok(())
}

#[test]
fn tagged_enums() -> Result<()> {
    let rt = Runtime::new(WASM_BYTES)?;
    assert_eq!(
        rt.export_fp_adjacently_tagged(FpAdjacentlyTagged::Bar("Hello, plugin!".to_owned()))?,
        FpAdjacentlyTagged::Baz { a: -8, b: 64 }
    );
    assert_eq!(
        rt.export_fp_internally_tagged(FpInternallyTagged::Foo)?,
        FpInternallyTagged::Baz { a: -8, b: 64 }
    );
    assert_eq!(
        rt.export_fp_untagged(FpUntagged::Bar("Hello, plugin!".to_owned()))?,
        FpUntagged::Baz { a: -8, b: 64 }
    );
    assert_eq!(
        rt.export_serde_adjacently_tagged(SerdeAdjacentlyTagged::Bar("Hello, plugin!".to_owned()))?,
        SerdeAdjacentlyTagged::Baz { a: -8, b: 64 }
    );
    assert_eq!(
        rt.export_serde_internally_tagged(SerdeInternallyTagged::Foo)?,
        SerdeInternallyTagged::Baz { a: -8, b: 64 }
    );
    assert_eq!(
        rt.export_serde_untagged(SerdeUntagged::Bar("Hello, plugin!".to_owned()))?,
        SerdeUntagged::Baz { a: -8, b: 64 }
    );
    Ok(())
}

#[tokio::test]
async fn async_struct() -> Result<()> {
    let rt = Runtime::new(WASM_BYTES)?;

    assert_eq!(
        rt.export_async_struct(
            FpPropertyRenaming {
                foo_bar: "foo_bar".to_owned(),
                qux_baz: 64.0,
                raw_struct: -32
            },
            64
        )
        .await?,
        FpPropertyRenaming {
            foo_bar: "fooBar".to_owned(),
            qux_baz: -64.0,
            raw_struct: 32,
        }
    );
    Ok(())
}

#[tokio::test]
async fn fetch_async_data() -> Result<()> {
    let rt = Runtime::new(WASM_BYTES)?;

    let response = rt.fetch_data("sign-up".to_string()).await?;

    assert_eq!(response, Ok(r#"status: "confirmed"#.to_string()));
    Ok(())
}

#[tokio::test]
async fn concurrent_delays() -> Result<()> {
    let rt = Runtime::new(WASM_BYTES)?;

    let responses = tokio::join!(
        rt.delay(true, 120),
        rt.delay(true, 90),
        rt.delay(false, 75),
        rt.delay(true, 60),
        rt.delay(true, 30),
    );
    assert_eq!(responses.0.ok().unwrap(), Ok(()));
    assert_eq!(responses.1.ok().unwrap(), Ok(()));
    assert_eq!(responses.2.ok().unwrap(), Err(()));
    assert_eq!(responses.3.ok().unwrap(), Ok(()));
    assert_eq!(responses.4.ok().unwrap(), Ok(()));

    Ok(())
}

#[test]
fn bytes() -> Result<()> {
    let rt = Runtime::new(WASM_BYTES)?;

    assert_eq!(rt.export_get_bytes()?, Ok(Bytes::from("hello, world")));
    assert_eq!(rt.export_get_serde_bytes()?, Ok(ByteBuf::from("hello, world")));

    Ok(())
}
