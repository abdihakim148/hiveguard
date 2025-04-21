use serde::de::{Deserializer, Deserialize, MapAccess, Visitor};
pub use super::{Key, Version};


impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de> {
        deserializer.deserialize_map(KeyVisitor)
    }
}


pub struct KeyVisitor;


impl<'de> Visitor<'de> for KeyVisitor {
    type Value = Key;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "expected a struct of private_key and public_key")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>, {
        let mut private_key = [0u8; 32];
        let mut public_key = [0u8; 32];
        let mut version = Version::default();

        while let Some(key) = map.next_key()? {    
            match key {
                "private_key" => {
                    private_key = map.next_value()?;
                },
                "public_key" => {
                    public_key = map.next_value()?;
                },
                "version" => {
                    version = map.next_value()?;
                }
                _ => ()
            }
        }

        let id = Key::id(&public_key);
        let key = Key{id, private_key, public_key, version};
        Ok(key)

    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialization() {
        let key = Key::default();
        let json = serde_json::to_string(&key).expect("This should Never happen");
        let new_key = serde_json::from_str(&json).unwrap();
        assert_eq!(key, new_key);
    }
}
