#![doc = include_str!("../README.md")]

pub mod account;
pub mod application;
pub mod client;
pub mod error;
pub mod infrastructure;
pub mod publisher;
#[cfg(feature = "test-utils")]
pub mod test_utils;

/// Not meant to be called directly
#[doc(hidden)]
#[macro_export]
macro_rules! __doc_setup_mock {
    ( ) => {
        $crate::client::AbstractClient::builder("sender")
            .build()
            .unwrap()
    };
}
