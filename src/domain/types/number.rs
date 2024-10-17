use std::convert::TryFrom; // Importing TryFrom trait for conversions
use std::fmt;
use crate::domain::types::{Error, ConversionError, Type};
use serde::{Serialize, Deserialize};

/// Enum representing various numerical types.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
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
                type Error = Error;

                fn try_from(number: Number) -> Result<Self, Self::Error> {
                    match number {
                        Number::$variant(value) if value >= <$t>::MIN as _ && value <= <$t>::MAX as _ => Ok(value as $t),
                        Number::U8(value) if value >= <$t>::MIN as u8 && value <= <$t>::MAX as u8 => Ok(value as $t),
                        Number::I8(value) if value >= <$t>::MIN as i8 && value <= <$t>::MAX as i8 => Ok(value as $t),
                        Number::U16(value) if value >= <$t>::MIN as u16 && value <= <$t>::MAX as u16 => Ok(value as $t),
                        Number::I16(value) if value >= <$t>::MIN as i16 && value <= <$t>::MAX as i16 => Ok(value as $t),
                        Number::U32(value) if value >= <$t>::MIN as u32 && value <= <$t>::MAX as u32 => Ok(value as $t),
                        Number::I32(value) if value >= <$t>::MIN as i32 && value <= <$t>::MAX as i32 => Ok(value as $t),
                        Number::U64(value) if value >= <$t>::MIN as u64 && value <= <$t>::MAX as u64 => Ok(value as $t),
                        Number::I64(value) if value >= <$t>::MIN as i64 && value <= <$t>::MAX as i64 => Ok(value as $t),
                        Number::U128(value) if value >= <$t>::MIN as u128 && value <= <$t>::MAX as u128 => Ok(value as $t),
                        Number::I128(value) if value >= <$t>::MIN as i128 && value <= <$t>::MAX as i128 => Ok(value as $t),
                        Number::F32(value) if value >= <$t>::MIN as f32 && value <= <$t>::MAX as f32 => Ok(value as $t),
                        Number::F64(value) if value >= <$t>::MIN as f64 && value <= <$t>::MAX as f64 => Ok(value as $t),
                        _ => Err(Error::ConversionError(format!("Expected {}, found {} of {}", stringify!($t), stringify!(number), number))),
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

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Number::U8(value) => write!(f, "{}", value),
            Number::I8(value) => write!(f, "{}", value),
            Number::U16(value) => write!(f, "{}", value),
            Number::I16(value) => write!(f, "{}", value),
            Number::U32(value) => write!(f, "{}", value),
            Number::I32(value) => write!(f, "{}", value),
            Number::U64(value) => write!(f, "{}", value),
            Number::I64(value) => write!(f, "{}", value),
            Number::U128(value) => write!(f, "{}", value),
            Number::I128(value) => write!(f, "{}", value),
            Number::F32(value) => write!(f, "{}", value),
            Number::F64(value) => write!(f, "{}", value),
        }
    }
}

impl From<Number> for Type {
    fn from(number: Number) -> Self {
        match number {
            Number::U8(_) => Type::U8,
            Number::I8(_) => Type::I8,
            Number::U16(_) => Type::U16,
            Number::I16(_) => Type::I16,
            Number::U32(_) => Type::U32,
            Number::I32(_) => Type::I32,
            Number::U64(_) => Type::U64,
            Number::I64(_) => Type::I64,
            Number::U128(_) => Type::U128,
            Number::I128(_) => Type::I128,
            Number::F32(_) => Type::F32,
            Number::F64(_) => Type::F64,
        }
    }
}


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
        let num = Number::U8(42); // Use a valid value for u8
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
        let num = Number::U32(1024);
        let result: Result<i8, _> = i8::try_from(num);
        assert!(result.is_err());
    }
}
