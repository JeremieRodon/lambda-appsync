use std::ops::Deref;

use serde::{Deserialize, Serialize};

/// A custom UUID-based identifier type for AppSync GraphQL objects.
///
/// This type wraps a UUIDv4 to provide a standardized way of identifying objects
/// in AppSync while ensuring type safety and validation. It implements serialization
/// and deserialization as [String] as expected by GraphQL.
///
/// # Example
/// ```
/// use lambda_appsync::ID;
///
/// let id = ID::new();
/// let id_str: String = id.into();
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct ID(uuid::Uuid);
impl ID {
    /// Create a new random ID based on the UUIDv4 specification.
    ///
    /// # Example
    /// ```
    /// use lambda_appsync::ID;
    ///
    /// let id = ID::new();
    /// ```
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}
impl Default for ID {
    fn default() -> Self {
        Self::new()
    }
}
impl TryFrom<String> for ID {
    type Error = uuid::Error;
    /// Attempts to create an ID from a string representation of a UUID.
    ///
    /// # Example
    /// ```
    /// use lambda_appsync::ID;
    ///
    /// let id = ID::try_from("123e4567-e89b-12d3-a456-426614174000".to_string()).unwrap();
    /// ```
    ///
    /// # Errors
    /// Returns a `uuid::Error` if:
    /// - The string is not a valid UUID format
    /// - The string contains invalid characters
    /// - The string is not of the correct length
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(ID(uuid::Uuid::parse_str(&value)?))
    }
}
impl core::fmt::Display for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl From<ID> for String {
    fn from(value: ID) -> Self {
        value.to_string()
    }
}
impl Deref for ID {
    type Target = uuid::Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_creation() {
        let id = ID::new();
        let id_str: String = id.into();

        // Test parsing back
        let parsed = ID::try_from(id_str.clone()).unwrap();
        assert_eq!(id, parsed);

        // Test display
        assert_eq!(id.to_string(), id_str);
    }

    #[test]
    fn test_new_id() {
        let id = ID::new();
        assert!(uuid::Uuid::parse_str(&id.to_string()).is_ok());
    }

    #[test]
    fn test_id_conversion() {
        let id = ID::new();
        let id_string = String::from(id);
        let converted_id = ID::try_from(id_string.clone()).unwrap();
        assert_eq!(id, converted_id);
        assert_eq!(id.to_string(), id_string);
    }

    #[test]
    fn test_invalid_id() {
        let result = ID::try_from("not-a-uuid".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_id_deref() {
        let id = ID::new();
        let uuid: &uuid::Uuid = id.deref();
        assert_eq!(uuid.to_string(), id.to_string());
    }

    #[test]
    fn test_id_display() {
        let id = ID::new();
        let uuid_string = id.0.to_string();
        assert_eq!(id.to_string(), uuid_string);
    }
}
