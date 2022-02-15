use rmp_serde::Serializer;
use serde::{de::DeserializeOwned, Serialize};
use serde_bytes::ByteBuf;

/// Serialize the given value to MessagePack
pub fn serialize_to_vec<T: Serialize>(value: &T) -> Vec<u8> {
    let mut buffer = Vec::new();
    let mut serializer = Serializer::new(&mut buffer)
        .with_struct_map()
        .with_human_readable();
    value.serialize(&mut serializer).unwrap();
    buffer
}

/// Deserialize the given MessagePack-encoded slice
pub fn deserialize_from_slice<T: DeserializeOwned>(
    slice: &[u8],
) -> Result<T, serde_path_to_error::Error<rmp_serde::decode::Error>> {
    let mut deserializer = rmp_serde::Deserializer::new(slice).with_human_readable();
    serde_path_to_error::deserialize(&mut deserializer)
}

pub fn serialize_to_byte_buf<T: Serialize>(value: &T) -> ByteBuf {
    ByteBuf::from(serialize_to_vec(value))
}
