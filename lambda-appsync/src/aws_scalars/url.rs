impl_new_string!(AWSUrl);

#[cfg(test)]
mod tests {
    use super::AWSUrl;

    #[test]
    fn url_from_string() {
        let value = String::from("https://example.com");
        let url = AWSUrl::from(value.clone());
        assert_eq!(*url, value);
    }

    #[test]
    fn url_from_str() {
        let value = "https://example.com";
        let url = AWSUrl::from(value);
        assert_eq!(*url, value);
    }

    #[test]
    fn url_into_string() {
        let value = "https://example.com";
        let url = AWSUrl::from(value);
        let string: String = url.into();
        assert_eq!(string, value);
    }

    #[test]
    fn url_display() {
        let value = "https://example.com";
        let url = AWSUrl::from(value);
        assert_eq!(url.to_string(), value);
    }
}
