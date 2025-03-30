use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

#[doc = "AWS AppSync specific GraphQL scalar type implemented a [String] new-type"]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(from = "u64", into = "u64")]
pub struct AWSTimestamp(SystemTime);
impl AWSTimestamp {
    pub fn now() -> Self {
        Self(SystemTime::now())
    }
}
impl From<AWSTimestamp> for u64 {
    fn from(value: AWSTimestamp) -> Self {
        value
            .0
            .duration_since(std::time::UNIX_EPOCH)
            .expect("we should never manipulate dates earlier than EPOCH")
            .as_secs()
    }
}
impl From<u64> for AWSTimestamp {
    fn from(value: u64) -> Self {
        Self(std::time::UNIX_EPOCH + Duration::from_secs(value))
    }
}
impl Default for AWSTimestamp {
    fn default() -> Self {
        Self::now()
    }
}
impl std::ops::Add<Duration> for AWSTimestamp {
    type Output = Self;
    fn add(self, rhs: Duration) -> Self::Output {
        Self(self.0 + rhs)
    }
}
impl std::ops::Sub<Duration> for AWSTimestamp {
    type Output = Self;
    fn sub(self, rhs: Duration) -> Self::Output {
        Self(self.0 - rhs)
    }
}
impl std::ops::Sub<AWSTimestamp> for AWSTimestamp {
    type Output = Duration;
    fn sub(self, rhs: AWSTimestamp) -> Self::Output {
        self.0
            .duration_since(rhs.0)
            .expect("the substracted AWSTimestamp MUST be earlier")
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_now() {
        let ts = AWSTimestamp::now();
        let now = SystemTime::now();
        let diff = now.duration_since(ts.0).unwrap();
        assert!(diff < Duration::from_secs(1));
    }

    #[test]
    fn test_timestamp_default() {
        let ts = AWSTimestamp::default();
        let now = SystemTime::now();
        let diff = now.duration_since(ts.0).unwrap();
        assert!(diff < Duration::from_secs(1));
    }

    #[test]
    fn test_timestamp_conversion() {
        let now = SystemTime::now();
        let secs = now.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let ts = AWSTimestamp::from(secs);
        let back_to_secs: u64 = ts.into();
        assert_eq!(secs, back_to_secs);
    }

    #[test]
    fn test_timestamp_add() {
        let ts = AWSTimestamp::from(1000);
        let ts2 = ts + Duration::from_secs(500);
        let secs: u64 = ts2.into();
        assert_eq!(secs, 1500);
    }

    #[test]
    fn test_timestamp_sub_duration() {
        let ts = AWSTimestamp::from(1000);
        let ts2 = ts - Duration::from_secs(500);
        let secs: u64 = ts2.into();
        assert_eq!(secs, 500);
    }

    #[test]
    fn test_timestamp_sub_timestamp() {
        let ts1 = AWSTimestamp::from(1500);
        let ts2 = AWSTimestamp::from(1000);
        let diff = ts1 - ts2;
        assert_eq!(diff.as_secs(), 500);
    }

    #[test]
    #[should_panic(expected = "the substracted AWSTimestamp MUST be earlier")]
    fn test_timestamp_sub_panic() {
        let ts1 = AWSTimestamp::from(1000);
        let ts2 = AWSTimestamp::from(1500);
        let _diff = ts1 - ts2;
    }
}
