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
