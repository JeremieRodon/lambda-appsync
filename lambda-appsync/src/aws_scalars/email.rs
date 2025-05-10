use crate::stdlib::alloc::string::String;

impl_new_string!(no_from AWSEmail);

// An Email address should always be lowercase
impl From<String> for AWSEmail {
    fn from(value: String) -> Self {
        // We could simply call `to_lowercase` and be done with it
        // but this way we avoid useless allocation be using the already
        // allocated String if possible
        if value.chars().all(|c| !c.is_uppercase()) {
            Self(value)
        } else {
            Self(value.to_lowercase())
        }
    }
}
impl From<&str> for AWSEmail {
    fn from(value: &str) -> Self {
        Self(value.to_lowercase())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::stdlib::alloc::string::ToString;

    #[test]
    fn email_lowercase() {
        let email = AWSEmail::from("TEST@EXAMPLE.COM");
        assert_eq!(*email, "test@example.com");
    }

    #[test]
    fn email_mixed_case() {
        let email = AWSEmail::from("Test@Example.com");
        assert_eq!(*email, "test@example.com");
    }

    #[test]
    fn email_already_lowercase() {
        let input = "test@example.com".to_string();
        let email = AWSEmail::from(input.clone());
        assert_eq!(*email, input);
    }

    #[test]
    fn email_from_str() {
        let email = AWSEmail::from("Test@Example.com");
        assert_eq!(*email, "test@example.com");
    }

    #[test]
    fn email_into_string() {
        let email = AWSEmail::from("test@example.com");
        let email_string: String = email.into();
        assert_eq!(email_string, "test@example.com");
    }
    #[test]
    fn email_display() {
        let value = "test@example.com";
        let email = AWSEmail::from(value);
        assert_eq!(email.to_string(), value);
    }
}
