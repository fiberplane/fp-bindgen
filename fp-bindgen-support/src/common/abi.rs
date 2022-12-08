pub trait WasmAbi {
    type AbiType;

    fn to_abi(self) -> Self::AbiType;
    fn from_abi(value: Self::AbiType) -> Self;
}

impl WasmAbi for bool {
    type AbiType = u8;

    #[inline]
    fn to_abi(self) -> Self::AbiType {
        u8::from(self)
    }

    #[inline]
    fn from_abi(value: Self::AbiType) -> Self {
        value != 0
    }
}

macro_rules! identity_wasm_abi {
    ($ty:ty) => {
        impl WasmAbi for $ty {
            type AbiType = $ty;

            #[inline]
            fn to_abi(self) -> Self::AbiType {
                self
            }

            #[inline]
            fn from_abi(value: Self::AbiType) -> Self {
                value
            }
        }
    };
    ($($ty:ty),*) => {
        $(
            identity_wasm_abi!($ty);
        )*
    }
}

identity_wasm_abi!((), u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);
