use std::str::FromStr;

/// Type of primitive that is supported out-of-the-box.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Primitive {
    Bool,
    F32,
    F64,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
}

impl Primitive {
    pub fn name(&self) -> String {
        use Primitive::*;
        let string = match self {
            Bool => "bool",
            F32 => "f32",
            F64 => "f64",
            I8 => "i8",
            I16 => "i16",
            I32 => "i32",
            I64 => "i64",
            I128 => "i128",
            U8 => "u8",
            U16 => "u16",
            U32 => "u32",
            U64 => "u64",
            U128 => "u128",
        };
        string.to_owned()
    }
}

impl FromStr for Primitive {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let primitive = match string {
            "bool" => Primitive::Bool,
            "f32" => Primitive::F32,
            "f64" => Primitive::F64,
            "i8" => Primitive::I8,
            "i16" => Primitive::I16,
            "i32" => Primitive::I32,
            "i64" => Primitive::I64,
            "i128" => Primitive::I128,
            "u8" => Primitive::U8,
            "u16" => Primitive::U16,
            "u32" => Primitive::U32,
            "u64" => Primitive::U64,
            "u128" => Primitive::U128,
            string => return Err(format!("Unknown primitive type: \"{}\"", string)),
        };
        Ok(primitive)
    }
}
