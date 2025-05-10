use crate::stdlib::alloc::{borrow::ToOwned, string::String};

impl_new_string!(AWSPhone);

#[cfg(test)]
mod tests {
    use super::AWSPhone;

    use crate::stdlib::alloc::string::{String, ToString};

    #[test]
    fn phone_from_string() {
        let value = String::from("+12345678901");
        let phone = AWSPhone::from(value.clone());
        assert_eq!(*phone, value);
    }

    #[test]
    fn phone_from_str() {
        let value = "+12345678901";
        let phone = AWSPhone::from(value);
        assert_eq!(*phone, value);
    }

    #[test]
    fn phone_into_string() {
        let value = "+12345678901";
        let phone = AWSPhone::from(value);
        let string: String = phone.into();
        assert_eq!(string, value);
    }

    #[test]
    fn phone_display() {
        let value = "+12345678901";
        let phone = AWSPhone::from(value);
        assert_eq!(phone.to_string(), value);
    }
}
