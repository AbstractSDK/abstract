mod error;
pub mod msg;
pub mod state;
pub mod utils;

pub use error::{contributors::ContributorsError, subscription::SubscriptionError};

pub const SUBSCRIPTION_ID: &str = "abstract:subscription";
pub const CONTRIBUTORS_ID: &str = "abstract:subscription-contributors";