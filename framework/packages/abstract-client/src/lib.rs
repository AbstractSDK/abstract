#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub(crate) mod account;
mod application;
pub mod builder;
mod client;
mod error;
pub(crate) mod infrastructure;
#[cfg(feature = "test-utils")]
mod mut_client;
mod publisher;
mod service;
pub(crate) mod source;

// Re-export common used types
pub use abstract_std::objects::{gov_type::GovernanceDetails, namespace::Namespace};
// Re-export `ClientResolve` trait
pub use abstract_interface::ClientResolve;
pub use account::{Account, AccountBuilder};
pub use application::Application;
pub use builder::AbstractClientBuilder;
pub use client::AbstractClient;
pub use error::AbstractClientError;
pub use infrastructure::Environment;
pub use publisher::{Publisher, PublisherBuilder};
pub use service::Service;
pub use source::AccountSource;

// Interchain stuff
#[cfg(feature = "interchain")]
mod interchain {
    pub(crate) mod remote_account;
    mod remote_application;
    use std::collections::HashMap;

    use cw_orch_interchain::{IbcQueryHandler, InterchainEnv, InterchainError};
    pub use remote_account::RemoteAccount;
    pub use remote_application::RemoteApplication;

    use crate::{client::AbstractClientResult, AbstractClient, Environment};

    // TODO: Why are we not returning ibc tx analysis after await
    /// IbcTxAnalysis after waiting for interchain action
    pub struct IbcTxAnalysisV2<Chain: cw_orch::environment::CwEnv>(
        pub cw_orch_interchain::types::IbcTxAnalysis<Chain>,
    );

    /// Client to interact with Abstract and setup interchain capabilities
    pub struct AbstractInterchainClient<Chain: IbcQueryHandler> {
        /// All clients registered within this Interchain Client
        pub clients: HashMap<String, AbstractClient<Chain>>,
    }

    impl<Chain: IbcQueryHandler> AbstractInterchainClient<Chain> {
        /// Deploys and connects Abstract instances across all chains specified
        pub fn build<Interchain: InterchainEnv<Chain>>(
            interchain: &Interchain,
        ) -> AbstractClientResult<Self> {
            // We deploy Abstract on all chains
            let clients = interchain
                .chains()
                .map_err(Into::into)?
                .iter()
                .map(|chain| AbstractClient::builder(chain.clone()).build())
                .collect::<Result<Vec<_>, _>>()?;

            // We connect all chains together
            for i in 0..clients.len() {
                for j in i + 1..clients.len() {
                    clients[i].connect_to(&clients[j], interchain)?;
                }
            }

            Ok(AbstractInterchainClient {
                clients: clients
                    .into_iter()
                    .map(|c| (c.environment().chain_id(), c))
                    .collect(),
            })
        }

        /// Getter for an abstract client within this object
        pub fn get_abstract(&self, chain_id: &str) -> AbstractClientResult<AbstractClient<Chain>> {
            self.clients
                .get(chain_id)
                .cloned()
                .ok_or(InterchainError::ChainNotFound(chain_id.to_string()))
                .map_err(Into::into)
        }
    }
}
#[cfg(feature = "interchain")]
pub use interchain::*;
