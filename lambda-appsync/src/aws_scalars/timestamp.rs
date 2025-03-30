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
