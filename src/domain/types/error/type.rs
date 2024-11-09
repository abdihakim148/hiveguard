use std::fmt::{Display, Formatter, Result};
use std::any::TypeId;
use Type::*;


#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    Usize,
    Isize,
    U128,
    I128,
    F32,
    F64,
    Bool,
    String,
    Vec(Box<Type>),
    Object(Box<(Type, Type)>),
    Option(Box<Type>),
    New(TypeId),
    Unknown
}


impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            U8 => write!(f, "u8"),
            I8 => write!(f, "i8"),
            U16 => write!(f, "u16"),
            I16 => write!(f, "i16"),
            U32 => write!(f, "u32"),
            I32 => write!(f, "i32"),
            U64 => write!(f, "u64"),
            I64 => write!(f, "i64"),
            Usize => write!(f,"uize"),
            Isize => write!(f, "isize"),
            U128 => write!(f, "u128"),
            I128 => write!(f, "i128"),
            F32 => write!(f, "f32"),
            F64 => write!(f, "f64"),
            Bool => write!(f, "bool"),
            String => write!(f, "String"),
            Vec(value) => write!(f, "Vec<{value}>"),
            Object(value) => write!(f, "Object<{}, {}>", value.0, value.1),
            Option(value) => write!(f, "Option<{value}>"),
            New(ty) => write!(f, "New<{ty:?}>"),
            Unknown => write!(f, "null")
        }
    }
}