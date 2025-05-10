use core::time::Duration;
use serde::{Deserialize, Serialize};

use web_time::{SystemTime, UNIX_EPOCH};

/// AWS AppSync specific GraphQL scalar type implemented [SystemTime] new-type.
/// Note that this type implements Copy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(from = "u64", into = "u64")]
pub struct AWSTimestamp(SystemTime);

impl AWSTimestamp {
    /// Returns an [AWSTimestamp] representing the current date and time, as reported by the system clock.
    ///
    /// # Example
    /// ```
    /// use lambda_appsync::AWSTimestamp;
    ///
    /// let now = AWSTimestamp::now();
    /// ```
    pub fn now() -> Self {
        Self(SystemTime::now())
    }

    /// Converts timestamp into UNIX epoch as number of seconds.
    ///
    /// # Examples
    /// ```
    /// use lambda_appsync::AWSTimestamp;
    ///
    /// let ts = AWSTimestamp::from(1234);
    /// assert_eq!(ts.into_u64(), 1234);
    /// ```
    pub fn into_u64(self) -> u64 {
        self.into()
    }

    /// Creates an [AWSTimestamp] from a u64 representing seconds since the UNIX epoch.
    ///
    /// # Examples
    /// ```
    /// use lambda_appsync::AWSTimestamp;
    ///
    /// let ts = AWSTimestamp::from_u64(1234);
    /// assert_eq!(ts.into_u64(), 1234);
    /// ```
    pub fn from_u64(value: u64) -> Self {
        Self::from(value)
    }
}

impl From<SystemTime> for AWSTimestamp {
    fn from(time: SystemTime) -> Self {
        Self(time)
    }
}

impl PartialEq<SystemTime> for AWSTimestamp {
    fn eq(&self, other: &SystemTime) -> bool {
        self.0 == *other
    }
}

impl From<AWSTimestamp> for u64 {
    fn from(value: AWSTimestamp) -> Self {
        value
            .0
            .duration_since(UNIX_EPOCH)
            .expect("we should never manipulate dates earlier than EPOCH")
            .as_secs()
    }
}

impl core::fmt::Display for AWSTimestamp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", u64::from(*self))
    }
}

impl From<u64> for AWSTimestamp {
    fn from(value: u64) -> Self {
        Self(UNIX_EPOCH + Duration::from_secs(value))
    }
}

impl Default for AWSTimestamp {
    fn default() -> Self {
        Self::now()
    }
}

impl core::ops::Add<Duration> for AWSTimestamp {
    type Output = Self;
    fn add(self, rhs: Duration) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl core::ops::AddAssign<Duration> for AWSTimestamp {
    fn add_assign(&mut self, rhs: Duration) {
        self.0 += rhs;
    }
}

impl core::ops::Sub<Duration> for AWSTimestamp {
    type Output = Self;
    fn sub(self, rhs: Duration) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl core::ops::SubAssign<Duration> for AWSTimestamp {
    fn sub_assign(&mut self, rhs: Duration) {
        self.0 -= rhs;
    }
}

impl core::ops::Sub<AWSTimestamp> for AWSTimestamp {
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
    use crate::stdlib::alloc::string::ToString;

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
        let secs = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
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
    fn test_timestamp_add_assign() {
        let mut ts = AWSTimestamp::from(1000);
        ts += Duration::from_secs(500);
        let secs: u64 = ts.into();
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
    fn test_timestamp_sub_assign() {
        let mut ts = AWSTimestamp::from(1000);
        ts -= Duration::from_secs(500);
        let secs: u64 = ts.into();
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

    #[test]
    fn test_display() {
        let ts = AWSTimestamp::from(1234);
        assert_eq!(ts.to_string(), "1234");
    }

    #[test]
    fn test_from_system_time() {
        let now = SystemTime::now();
        let ts = AWSTimestamp::from(now);
        assert_eq!(ts.0, now);
    }

    #[test]
    fn test_partial_eq_system_time() {
        let now = SystemTime::now();
        let ts = AWSTimestamp::from(now);
        assert_eq!(ts, now);
    }

    #[test]
    fn test_into_u64() {
        let ts = AWSTimestamp::from(1234);
        assert_eq!(ts.into_u64(), 1234);
    }

    #[test]
    fn test_from_u64() {
        let ts = AWSTimestamp::from_u64(1234);
        assert_eq!(ts.into_u64(), 1234);
    }
}
