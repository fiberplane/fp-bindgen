/// Type of primitive that is supported out-of-the-box.
pub enum Primitive {
    Bool,
    F32,
    F64,
    I8,
    I16,
    I32,
    I64,
    I128,
    Str,
    String,
    U8,
    U16,
    U32,
    U64,
    U128,
    Unit,
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
            Str => "&'static str",
            String => "String",
            U8 => "u8",
            U16 => "u16",
            U32 => "u32",
            U64 => "u64",
            U128 => "u128",
            Unit => "()",
        };
        string.to_owned()
    }
}
