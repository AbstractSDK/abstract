pub(crate) mod remote_account;
mod remote_application;
use std::collections::HashMap;

use abstract_interface::Abstract;
use cosmwasm_std::Addr;
use cw_orch_interchain::prelude::*;
pub use remote_account::RemoteAccount;
pub use remote_application::RemoteApplication;

use crate::{client::AbstractClientResult, AbstractClient, Environment};

/// Client to interact with Abstract and setup interchain capabilities
pub struct AbstractInterchainClient<Chain: IbcQueryHandler> {
    /// All clients registered within this Interchain Client
    pub clients: HashMap<String, AbstractClient<Chain>>,
}

impl<Chain: IbcQueryHandler> AbstractInterchainClient<Chain> {
    /// Loads Abstract from all the environments specified inside `interchain_env`
    /// Use [`AbstractInterchainClient::client`] to get a single abstract instance
    pub fn new<Interchain: InterchainEnv<Chain>>(
        interchain: &Interchain,
    ) -> AbstractClientResult<Self> {
        let clients = interchain
            .chains()
            .map(|chain| AbstractClient::new(chain.clone()))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(AbstractInterchainClient {
            clients: clients
                .into_iter()
                .map(|c| (c.environment().chain_id(), c))
                .collect(),
        })
    }

    /// Loads Abstract from all the environments specified inside `interchain_env`
    /// Use [`AbstractInterchainClient::client`] to get a single abstract instance
    pub fn load_from<Interchain: InterchainEnv<Chain>>(
        interchain: &Interchain,
    ) -> AbstractClientResult<Self> {
        Self::new(interchain)
    }

    /// Deploys and connects Abstract instances across all chains specified
    /// Use [`AbstractInterchainClient::client`] to get a single abstract instance
    pub fn deploy_on<Interchain: InterchainEnv<Chain>>(
        interchain: &Interchain,
    ) -> AbstractClientResult<Self> {
        // We deploy Abstract on all chains
        let clients = interchain
            .chains()
            .map(|chain| AbstractClient::builder(chain.clone()).build(chain.sender().clone()))
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
    pub fn client(&self, chain_id: &str) -> AbstractClientResult<AbstractClient<Chain>> {
        self.clients
            .get(chain_id)
            .cloned()
            .ok_or(InterchainError::ChainNotFound(chain_id.to_string()))
            .map_err(Into::into)
    }
}
