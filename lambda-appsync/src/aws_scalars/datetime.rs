use crate::stdlib::alloc::{borrow::ToOwned, string::String};

impl_new_string!(AWSDateTime);
impl_new_string!(AWSDate);
impl_new_string!(AWSTime);

#[cfg(test)]
mod test {
    use super::*;

    use crate::stdlib::alloc::string::ToString;

    #[test]
    fn datetime_from_str() {
        let dt = AWSDateTime::from("2024-02-14T15:30:00Z");
        assert_eq!(*dt, "2024-02-14T15:30:00Z");
    }

    #[test]
    fn datetime_into_string() {
        let dt = AWSDateTime::from("2024-02-14T15:30:00Z");
        let s: String = dt.into();
        assert_eq!(s, "2024-02-14T15:30:00Z");
    }

    #[test]
    fn date_from_str() {
        let d = AWSDate::from("2024-02-14");
        assert_eq!(*d, "2024-02-14");
    }

    #[test]
    fn date_into_string() {
        let d = AWSDate::from("2024-02-14");
        let s: String = d.into();
        assert_eq!(s, "2024-02-14");
    }

    #[test]
    fn time_from_str() {
        let t = AWSTime::from("15:30:00");
        assert_eq!(*t, "15:30:00");
    }

    #[test]
    fn time_into_string() {
        let t = AWSTime::from("15:30:00");
        let s: String = t.into();
        assert_eq!(s, "15:30:00");
    }

    #[test]
    fn display_implementations() {
        let dt = AWSDateTime::from("2024-02-14T15:30:00Z");
        let d = AWSDate::from("2024-02-14");
        let t = AWSTime::from("15:30:00");

        assert_eq!(dt.to_string(), "2024-02-14T15:30:00Z");
        assert_eq!(d.to_string(), "2024-02-14");
        assert_eq!(t.to_string(), "15:30:00");
    }
}
