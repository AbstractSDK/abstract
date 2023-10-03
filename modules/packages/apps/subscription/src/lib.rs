pub mod contributors;
pub mod subscription;
pub mod utils;

pub use contributors::error::ContributorsError;
pub use subscription::error::SubscriptionError;

pub const SUBSCRIPTION_ID: &str = "abstract:subscription";
pub const CONTRIBUTORS_ID: &str = "abstract:subscription-contributors";

/// Duration of subscription in weeks
pub const DURATION_IN_WEEKS: u64 = 4;
pub const WEEK_IN_SECONDS: u64 = 7 * 24 * 60 * 60;
