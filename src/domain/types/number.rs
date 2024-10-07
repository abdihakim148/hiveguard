use std::convert::TryFrom;
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



macro_rules! impl_from_for_number {
    ($($t:ty => $variant:ident),*) => {
        $(
            impl From<$t> for Number {
                fn from(value: $t) -> Self {
                    Number::$variant(value)
                }
            }

            impl TryFrom<Number> for $t {
                type Error = &'static str;

                fn try_from(number: Number) -> Result<Self, Self::Error> {
                    if let Number::$variant(value) = number {
                        Ok(value)
                    } else {
                        Err("Invalid conversion from Number")
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn test_from_primitive_to_number() {
        let num_u8: Number = 42u8.into();
        assert_eq!(num_u8, Number::U8(42));

        let num_i32: Number = (-42i32).into();
        assert_eq!(num_i32, Number::I32(-42));

        let num_f64: Number = 42.0f64.into();
        assert_eq!(num_f64, Number::F64(42.0));
    }

    #[test]
    fn test_try_from_number_to_primitive() {
        let num = Number::U8(42);
        let result: Result<u8, _> = u8::try_from(num);
        assert_eq!(result, Ok(42));

        let num = Number::I32(-42);
        let result: Result<i32, _> = i32::try_from(num);
        assert_eq!(result, Ok(-42));

        let num = Number::F64(42.0);
        let result: Result<f64, _> = f64::try_from(num);
        assert_eq!(result, Ok(42.0));
    }

    #[test]
    fn test_invalid_try_from_number() {
        let num = Number::U8(42);
        let result: Result<i32, _> = i32::try_from(num);
        assert!(result.is_err());
    }
}
