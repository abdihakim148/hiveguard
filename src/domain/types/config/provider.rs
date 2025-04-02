use oauth2::{ClientId, ClientSecret, AuthUrl, TokenUrl, Scope, EndpointSet, EndpointNotSet, basic};
use serde::{Deserialize, Serialize, Deserializer, Serializer};
use serde::de::{self, Visitor, MapAccess};
use crate::domain::types::user::User;
use serde::ser::{SerializeStruct};
use std::fmt;
use url::Url;


pub type BasicClient<T1 = EndpointNotSet, T2 = EndpointNotSet, T3 = EndpointNotSet> = basic::BasicClient<EndpointSet, T1, T2, T3, EndpointSet>;

/// OAuth Provider configuration for third-party authentication
#[derive(Debug, Clone)]
pub struct Provider {
    /// The OAuth client with all credentials
    pub client: BasicClient,
    
    /// OAuth scopes required for authentication
    pub scopes: Vec<Scope>
}

// Custom deserialization implementation for Provider
impl<'de> Deserialize<'de> for Provider {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Define the fields we expect in the serialized format
        enum Field { ClientId, ClientSecret, AuthUrl, TokenUrl, Scopes }

        // Implement visitor for deserializing Field enum
        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`client_id`, `client_secret`, `auth_url`, `token_url`, or `scopes`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "client_id" => Ok(Field::ClientId),
                            "client_secret" => Ok(Field::ClientSecret),
                            "auth_url" => Ok(Field::AuthUrl),
                            "token_url" => Ok(Field::TokenUrl),
                            "scopes" => Ok(Field::Scopes),
                            _ => Err(de::Error::unknown_field(value, &["client_id", "client_secret", "auth_url", "token_url", "scopes"])),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        // Visitor struct for deserializing the Provider
        struct ProviderVisitor;

        impl<'de> Visitor<'de> for ProviderVisitor {
            type Value = Provider;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Provider")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Provider, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut client_id = None;
                let mut client_secret = None;
                let mut auth_url = None;
                let mut token_url = None;
                let mut string_scopes = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::ClientId => {
                            if client_id.is_some() {
                                return Err(de::Error::duplicate_field("client_id"));
                            }
                            client_id = Some(map.next_value()?);
                        }
                        Field::ClientSecret => {
                            if client_secret.is_some() {
                                return Err(de::Error::duplicate_field("client_secret"));
                            }
                            client_secret = Some(map.next_value()?);
                        }
                        Field::AuthUrl => {
                            if auth_url.is_some() {
                                return Err(de::Error::duplicate_field("auth_url"));
                            }
                            auth_url = Some(map.next_value()?);
                        }
                        Field::TokenUrl => {
                            if token_url.is_some() {
                                return Err(de::Error::duplicate_field("token_url"));
                            }
                            token_url = Some(map.next_value()?);
                        }
                        Field::Scopes => {
                            if string_scopes.is_some() {
                                return Err(de::Error::duplicate_field("scopes"));
                            }
                            string_scopes = Some(map.next_value()?);
                        }
                    }
                }

                // Extract the required fields
                let client_id: String = client_id.ok_or_else(|| de::Error::missing_field("client_id"))?;
                let client_secret: String = client_secret.ok_or_else(|| de::Error::missing_field("client_secret"))?;
                let auth_url: Url = auth_url.ok_or_else(|| de::Error::missing_field("auth_url"))?;
                let token_url: Url = token_url.ok_or_else(|| de::Error::missing_field("token_url"))?;
                let string_scopes: Vec<String> = string_scopes.ok_or_else(|| de::Error::missing_field("scopes"))?;

                // Construct the BasicClient
                let client = basic::BasicClient::new(ClientId::new(client_id))
                    .set_client_secret(ClientSecret::new(client_secret))
                    .set_auth_uri(AuthUrl::from_url(auth_url))
                    .set_token_uri(TokenUrl::from_url(token_url));

                // Convert scopes from String to Scope
                let scopes = string_scopes
                    .into_iter()
                    .map(Scope::new)
                    .collect::<Vec<Scope>>();

                Ok(Provider { client, scopes })
            }
        }

        // Deserialize using our visitor
        deserializer.deserialize_struct(
            "Provider", 
            &["client_id", "client_secret", "auth_url", "token_url", "scopes"], 
            ProviderVisitor
        )
    }
}

// Custom serialization implementation for Provider using OAuth2 crate methods
impl Serialize for Provider {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Extract client ID using the client_id() method
        let client_id = self.client.client_id().as_str();

        // Extract client secret using the client_secret() method
        let client_secret = "$CLIENT_SECRET";

        // Extract auth URL using the auth_uri() method
        let auth_url = self.client.auth_uri().as_str();

        // Extract token URL using the token_uri() method
        let token_url = self.client.token_uri().as_str();

        // Convert Scope objects back to strings
        let scopes: Vec<&str> = self.scopes
            .iter()
            .map(|scope| scope.as_str())
            .collect();

        // Create a struct with 5 fields
        let mut state = serializer.serialize_struct("Provider", 5)?;
        state.serialize_field("client_id", client_id)?;
        state.serialize_field("client_secret", client_secret)?;
        state.serialize_field("auth_url", auth_url)?;
        state.serialize_field("token_url", token_url)?;
        state.serialize_field("scopes", &scopes)?;
        state.end()
    }
}

impl From<&Provider> for BasicClient {
    fn from(provider: &Provider) -> Self {
        provider.client.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, to_string, from_str};

    #[test]
    fn test_provider_deserialization() {
        let json_data = r#"{
            "client_id": "test_client_id",
            "client_secret": "test_client_secret",
            "auth_url": "https://auth.example.com/oauth",
            "token_url": "https://api.example.com/token",
            "scopes": ["read", "write", "profile"]
        }"#;

        let provider: Provider = from_str(json_data).expect("Failed to deserialize Provider");
        
        // Verify client data was properly extracted using accessor methods
        assert_eq!(provider.client.client_id().as_str(), "test_client_id");
        assert_eq!(provider.client.auth_uri().as_str(), "https://auth.example.com/oauth");
        assert_eq!(provider.client.token_uri().as_str(), "https://api.example.com/token");
        
        // Verify scopes were properly converted
        assert_eq!(provider.scopes.len(), 3);
        assert_eq!(provider.scopes[0].as_str(), "read");
        assert_eq!(provider.scopes[1].as_str(), "write");
        assert_eq!(provider.scopes[2].as_str(), "profile");
    }

    #[test]
    fn test_provider_serialization() {
        // Create a Provider instance programmatically
        let client = basic::BasicClient::new(ClientId::new("test_client_id".into()))
            .set_client_secret(ClientSecret::new("test_client_secret".into()))
            .set_auth_uri(AuthUrl::new("https://auth.example.com/oauth".to_string()).unwrap())
            .set_token_uri(TokenUrl::new("https://api.example.com/token".to_string()).unwrap());
        
        let scopes = vec![
            Scope::new("read".to_string()),
            Scope::new("write".to_string()),
            Scope::new("profile".to_string())
        ];
        
        let provider = Provider { client, scopes };
        
        // Serialize to JSON
        let serialized = to_string(&provider).expect("Failed to serialize Provider");
        
        // Parse the JSON and verify the structure
        let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(parsed["client_id"], "test_client_id");
        assert_eq!(parsed["client_secret"], "test_client_secret");
        assert_eq!(parsed["auth_url"], "https://auth.example.com/oauth");
        assert_eq!(parsed["token_url"], "https://api.example.com/token");
        
        let scopes = parsed["scopes"].as_array().unwrap();
        assert_eq!(scopes.len(), 3);
        assert_eq!(scopes[0], "read");
        assert_eq!(scopes[1], "write");
        assert_eq!(scopes[2], "profile");
    }

    #[test]
    fn test_roundtrip_serialization() {
        // Original JSON data
        let original_json = r#"{
            "client_id": "test_client_id",
            "client_secret": "test_client_secret",
            "auth_url": "https://auth.example.com/oauth",
            "token_url": "https://api.example.com/token",
            "scopes": ["read", "write", "profile"]
        }"#;

        // Deserialize
        let provider: Provider = from_str(original_json).expect("Failed to deserialize Provider");
        
        // Serialize back to JSON
        let serialized = to_string(&provider).expect("Failed to serialize Provider");
        
        // Normalize both JSONs for comparison (remove whitespace, etc.)
        let normalized_original: serde_json::Value = serde_json::from_str(original_json).unwrap();
        let normalized_serialized: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        
        // Compare the normalized JSON values
        assert_eq!(normalized_original, normalized_serialized);
    }
}