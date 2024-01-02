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

// #[allow(dead_code)]
// // https://github.com/rust-lang/rust/issues/67295
// pub fn __doc_setup_daemon(
//     handle: &cw_orch::tokio::runtime::Handle,
// ) -> client::AbstractClient<cw_orch::daemon::Daemon> {
//     use ::cw_orch::daemon::{networks::parse_network, Daemon};
//     ::std::env::set_var("CW_ORCH_DISABLE_ENABLE_LOGS_MESSAGE", "true");

//     // https://github.com/CosmosContracts/juno/blob/65fe9073e4e83afeb64c37a1acb5d3acb7d90876/docker/test-user.env
//     const TEST_MNEMONIC: &str ="clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose";
//     let juno_testnet = Daemon::builder()
//         .handle(handle)
//         .chain(parse_network("uni-6").unwrap())
//         // TODO: remove when daemon without signer is allowed
//         .mnemonic(TEST_MNEMONIC)
//         .build()
//         .unwrap();
//     client::AbstractClient::new(juno_testnet).unwrap()
// }
