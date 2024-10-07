use std::convert::TryFrom;

macro_rules! impl_from_for_number {
    ($($t:ty => $variant:ident),*) => {
        $(
            impl From<$t> for Number {
                fn from(value: $t) -> Self {
                    Number::$variant(value)
                }
            }

            impl From<Number> for $t {
                fn from(number: Number) -> Self {
                    if let Number::$variant(value) = number {
                        value
                    } else {
                        panic!("Invalid conversion from Number to {}", stringify!($t));
                    }
                }
            }
        )*
    };
}

impl_from_for_number!(
    u8 => U8,
    i8 => I8,
    u16 => U16,
    i16 => I16,
    u32 => U32,
    i32 => I32,
    u64 => U64,
    i64 => I64,
    u128 => U128,
    i128 => I128,
    f32 => F32,
    f64 => F64
);
use serde::{Serialize, Deserialize};

/// Enum representing various numerical types.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Number {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    U128(u128),
    I128(i128),
    F32(f32),
    F64(f64),
}
