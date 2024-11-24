use argon2::{Algorithm, Version, Params, Argon2};
use std::collections::HashMap;
use super::{Value, Number};
use static_init::dynamic;

// #[dynamic]
static mut PEPPER: Option<String> = None;
#[dynamic]
static mut ALGORITHM: Algorithm = Default::default();
#[dynamic]
static mut VERSION: Version = Default::default();

#[dynamic]
pub static mut ARGON2: Argon2<'static> = Argon2::default();


impl From<Value> for Version {
    fn from(value: Value) -> Self {
        match value {
            Value::Number(number) => {
                match TryInto::<u8>::try_into(number) {
                    Err(_) => Version::default(),
                    Ok(number) => {
                        match number {
                            16 => Version::V0x10,
                            18 => Version::V0x13,
                            _ => Version::default()
                        }
                    }
                }
            },
            _ => Version::default()
        }
    }
}

impl From<Version> for Value {
    fn from(version: Version) -> Self {
        match version {
            Version::V0x10 => Value::Number(Number::U8(16)),
            Version::V0x13 => Value::Number(Number::U8(19))
        }
    }
}



impl From<Value> for Algorithm {
    fn from(value: Value) -> Self {
        let algorithm = match value {
            Value::String(value) => {
                let string = value.to_lowercase();
                match string.as_str() {
                    "argon2d" => Algorithm::Argon2d,
                    "argon2i" => Algorithm::Argon2i,
                    "argon2id" | "argon2di" => Algorithm::Argon2id,
                    _ => Default::default()
                }
            },
            _ => Algorithm::default()
        };
        *ALGORITHM.write() = algorithm;
        algorithm
    }
}


impl From<Algorithm> for Value {
    fn from(algorithm: Algorithm) -> Self {
        match algorithm {
            Algorithm::Argon2d => Value::String(String::from("argon2d")),
            Algorithm::Argon2i => Value::String(String::from("argon2i")),
            Algorithm::Argon2id => Value::String(String::from("argon2id"))
        }
    }
}


impl From<Value> for Params {
    fn from(value: Value) -> Self {
        match value {
            Value::Object(mut map) => {
                let default = Params::default();
                let m_cost = if let Some(value) = map.remove("m_cost") {if let Ok(v) = TryInto::<(u32,)>::try_into(value) {v.0} else {default.m_cost()}} else {default.m_cost()};
                let t_cost = if let Some(value) = map.remove("t_cost") {if let Ok(v) = TryInto::<(u32,)>::try_into(value) {v.0} else {default.t_cost()}} else {default.m_cost()};
                let p_cost = if let Some(value) = map.remove("p_cost") {if let Ok(v) = TryInto::<(u32,)>::try_into(value) {v.0} else {default.p_cost()}} else {default.m_cost()};
                let output_len = if let Some(value) = map.remove("output_len") {if let Ok(tuple) = TryInto::<(usize,)>::try_into(value) {Some(tuple.0)} else {None}} else {default.output_len()};
                Params::new(m_cost, t_cost, p_cost, output_len).unwrap_or_default()
            },
            _ => Params::default()
        }
    }
}


impl From<Params> for Value {
    fn from(params: Params) -> Self {
        let mut map = HashMap::new();
        map.insert("m_cost".into(), Value::Number(Number::U32(params.m_cost())));
        map.insert("t_cost".into(), Value::Number(Number::U32(params.t_cost())));
        map.insert("p_cost".into(), Value::Number(Number::U32(params.p_cost())));
        if let Some(len) = params.output_len() {
            map.insert("output_len".into(), Value::Number(Number::Usize(len)));
        }else {
            map.insert("output_len".into(), Value::None);
        }
        Value::Object(map)
    }
}

/// This function executes unsafe code if executed in a multithreaded environment.
impl From<Value> for Argon2<'static> {
    fn from(value: Value) -> Self {
        match value {
            Value::Object(mut map) => {
                let algorithm = if let Some(value) = map.remove("algorithm") {value.into()} else {Algorithm::default()};
                *ALGORITHM.write() = algorithm;
                let version = if let Some(value) = map.remove("version") {value.into()} else {Version::default()};
                *VERSION.write() = version;
                let params = if let Some(value) = map.remove("params") {value.into()} else {Params::default()};
                let secret = if let Some(value) = map.remove("pepper") {if let Ok(string) = String::try_from(value){Some(string)} else {None}} else {None};
                match secret {
                    None => Argon2::new(algorithm, version, params),
                    // This contains unsafe code if used in a multithreaded environment.
                    Some(value) => {
                        unsafe {PEPPER = Some(value)};
                        let secret = unsafe {PEPPER.as_ref().unwrap().as_bytes()};
                        Argon2::new_with_secret(secret, algorithm, version, params).unwrap_or_default()
                    }
                }
            },
            _ => Default::default()
        }
    }
}


/// This function executes unsafe code if executed in a multithreaded environment.
impl From<Argon2<'static>> for Value {
    fn from(argon: Argon2<'static>) -> Self {
        let mut map = HashMap::new();
        let algorithm = (*(ALGORITHM.read())).clone().into();
        let version = (*(VERSION.read())).clone().into();
        let params = argon.params().clone().into();
        let secret = match unsafe {PEPPER.as_ref()} {Some(value) => Value::String(value.clone()), None => Value::None};
        map.insert("algorithm".into(), algorithm);
        map.insert("version".into(), version);
        map.insert("params".into(), params);
        map.insert("pepper".into(), secret);
        Value::Object(map)
    }
}