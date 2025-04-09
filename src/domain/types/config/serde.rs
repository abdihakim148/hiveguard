// Comprehensive macro that implements Serialize, Deserialize and tests
#[macro_export]
macro_rules! impl_oauth_provider_serde {
    ($provider_type:ty, $struct_name:expr) => {
        // Serialization/Deserialization implementation
        impl<'de> Deserialize<'de> for $provider_type {
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
                    type Value = $provider_type;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str(concat!("struct ", $struct_name))
                    }

                    fn visit_map<V>(self, mut map: V) -> Result<$provider_type, V::Error>
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
                            .set_client_secret(ClientSecret::new(client_secret.clone()))
                            .set_auth_uri(AuthUrl::from_url(auth_url))
                            .set_token_uri(TokenUrl::from_url(token_url));

                        // Convert scopes from String to Scope
                        let scopes = string_scopes
                            .into_iter()
                            .map(Scope::new)
                            .collect::<Vec<Scope>>();

                        Ok(Self::Value{client, client_secret, scopes})
                    }
                }

                // Deserialize using our visitor
                deserializer.deserialize_struct(
                    $struct_name, 
                    &["client_id", "client_secret", "auth_url", "token_url", "scopes"], 
                    ProviderVisitor
                )
            }
        }

        impl Serialize for $provider_type {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                // Extract client ID using the client_id() method
                let client_id = self.client.client_id().as_str();

                // Extract client secret using the client_secret() method
                let client_secret = self.client_secret.as_str();

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
                let mut state = serializer.serialize_struct($struct_name, 5)?;
                state.serialize_field("client_id", client_id)?;
                state.serialize_field("client_secret", client_secret)?;
                state.serialize_field("auth_url", auth_url)?;
                state.serialize_field("token_url", token_url)?;
                state.serialize_field("scopes", &scopes)?;
                state.end()
            }
        }

        // Implement From trait for test helpers
        impl From<(BasicClient, String, Vec<Scope>)> for $provider_type {
            fn from(parts: (BasicClient, String, Vec<Scope>)) -> Self {
                Self {
                    client: parts.0,
                    client_secret: parts.1,
                    scopes: parts.2,
                }
            }
        }

        // Generate test module
        #[cfg(test)]
        mod tests {
            use super::*;
            use serde_json::{json, to_string, from_str};

            // Helper function to create test provider
            fn create_test_instance() -> $provider_type {
                let client_id = "test_client_id";
                let client_secret = "test_client_secret";
                let auth_url = Url::parse("https://example.com/auth").unwrap();
                let token_url = Url::parse("https://example.com/token").unwrap();
                let scopes = vec![
                    Scope::new("email".to_string()),
                    Scope::new("profile".to_string())
                ];
                
                let client = basic::BasicClient::new(ClientId::new(client_id.to_string()))
                    .set_client_secret(ClientSecret::new(client_secret.to_string()))
                    .set_auth_uri(AuthUrl::from_url(auth_url))
                    .set_token_uri(TokenUrl::from_url(token_url));
                
                <$provider_type>::from((client, client_secret.to_string(), scopes))
            }

            #[test]
            fn test_serialization() {
                let provider = create_test_instance();
                
                let serialized = to_string(&provider).expect(&format!("Failed to serialize {}", $struct_name));
                let json_value: serde_json::Value = serde_json::from_str(&serialized).expect("Not valid JSON");
                
                assert!(json_value.get("client_id").is_some());
                assert_eq!(json_value["client_id"], "test_client_id");
                
                assert!(json_value.get("client_secret").is_some());
                assert_eq!(json_value["client_secret"], "test_client_secret");
                
                assert!(json_value.get("auth_url").is_some());
                assert_eq!(json_value["auth_url"], "https://example.com/auth");
                
                assert!(json_value.get("token_url").is_some());
                assert_eq!(json_value["token_url"], "https://example.com/token");
                
                assert!(json_value.get("scopes").is_some());
                let scopes = json_value["scopes"].as_array().expect("scopes should be an array");
                assert_eq!(scopes.len(), 2);
                assert!(scopes.contains(&json!("email")));
                assert!(scopes.contains(&json!("profile")));
            }

            #[test]
            fn test_deserialization() {
                let json_data = r#"{
                    "client_id": "test_client_id",
                    "client_secret": "test_client_secret",
                    "auth_url": "https://example.com/auth",
                    "token_url": "https://example.com/token",
                    "scopes": ["email", "profile"]
                }"#;
                
                let provider: $provider_type = from_str(json_data).expect(&format!("Failed to deserialize {}", $struct_name));
                
                // Verify the fields
                assert_eq!(provider.client.client_id().as_str(), "test_client_id");
                assert_eq!(provider.client_secret, "test_client_secret");
                assert_eq!(provider.client.auth_uri().as_str(), "https://example.com/auth");
                assert_eq!(provider.client.token_uri().as_str(), "https://example.com/token");
                
                // Verify scopes
                assert_eq!(provider.scopes.len(), 2);
                assert!(provider.scopes.iter().any(|s| s.as_str() == "email"));
                assert!(provider.scopes.iter().any(|s| s.as_str() == "profile"));
            }

            #[test]
            fn test_round_trip() {
                let original = create_test_instance();
                
                // Serialize
                let serialized = to_string(&original).expect(&format!("Failed to serialize {}", $struct_name));
                
                // Deserialize
                let deserialized: $provider_type = from_str(&serialized).expect(&format!("Failed to deserialize {}", $struct_name));
                
                // Verify round trip
                assert_eq!(deserialized.client.client_id().as_str(), "test_client_id");
                assert_eq!(deserialized.client_secret, "test_client_secret");
                assert_eq!(deserialized.client.auth_uri().as_str(), "https://example.com/auth");
                assert_eq!(deserialized.client.token_uri().as_str(), "https://example.com/token");
                assert_eq!(deserialized.scopes.len(), 2);
            }

            #[test]
            fn test_missing_fields() {
                // Missing client_id
                let json_data = r#"{
                    "client_secret": "test_client_secret",
                    "auth_url": "https://example.com/auth",
                    "token_url": "https://example.com/token",
                    "scopes": ["email", "profile"]
                }"#;
                
                let result: Result<$provider_type, _> = from_str(json_data);
                assert!(result.is_err(), "Should fail with missing client_id");
                
                // Missing scopes
                let json_data = r#"{
                    "client_id": "test_client_id",
                    "client_secret": "test_client_secret",
                    "auth_url": "https://example.com/auth",
                    "token_url": "https://example.com/token"
                }"#;
                
                let result: Result<$provider_type, _> = from_str(json_data);
                assert!(result.is_err(), "Should fail with missing scopes");
            }

            #[test]
            fn test_invalid_url() {
                // Invalid auth_url
                let json_data = r#"{
                    "client_id": "test_client_id",
                    "client_secret": "test_client_secret",
                    "auth_url": "not-a-valid-url",
                    "token_url": "https://example.com/token",
                    "scopes": ["email", "profile"]
                }"#;
                
                let result: Result<$provider_type, _> = from_str(json_data);
                assert!(result.is_err(), "Should fail with invalid URL");
            }
        }
    };
}