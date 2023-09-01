//! # Abstract Api Base
//!
//! `abstract_core::adapter` implements shared functionality that's useful for creating new Abstract adapters.
//!
//! ## Description
//! An Abstract adapter contract is a contract that is allowed to perform actions on a [proxy](crate::proxy) contract.
//! It is not migratable and its functionality is shared between users, meaning that all users call the same contract address to perform operations on the Account.
//! The api structure is well-suited for implementing standard interfaces to external services like dexes, lending platforms, etc.

use crate::{
    manager,
    objects::{account::AccountId, chain_name::ChainName},
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Binary, CosmosMsg};

pub mod state {
    use cw_storage_plus::{Item, Map};

    use crate::objects::ans_host::AnsHost;

    use super::*;

    /// Store channel information for account creation reply
    pub const REGISTRATION_CACHE: Item<AccountId> = Item::new("rc");
    /// account_id -> client_proxy_addr
    pub const CLIENT_PROXY: Map<&AccountId, String> = Map::new("cp");
    /// Maps a chain name to the proxy it uses to interact on this local chain
    pub const CHAIN_PROXYS: Map<&ChainName, Addr> = Map::new("ccl");
    pub const REVERSE_CHAIN_PROXYS: Map<&Addr, ChainName> = Map::new("reverse-ccl");
    /// Configuration of the IBC host
    pub const CONFIG: Item<'static, Config> = Item::new("cfg");


    /// The BaseState contains the main addresses needed for sending and verifying messages
    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        /// AnsHost contract struct (address)
        pub ans_host: AnsHost,
        /// Address of the account factory, used to create remote accounts
        pub account_factory: Addr,
        /// Address of the local version control, for retrieving account information
        pub version_control: Addr,
    }

}
/// Used by Abstract to instantiate the contract
/// The contract is then registered on the version control contract using [`crate::version_control::ExecuteMsg::ProposeModules`].
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    /// Used to easily perform address translation on the app chain
    pub ans_host_address: String,
    /// Used to create remote abstract accounts
    pub account_factory_address: String,
    /// Version control address
    pub version_control_address: String,
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}

#[cosmwasm_schema::cw_serde]
pub enum InternalAction {
    /// Registers a new account on the remote chain
    Register {
        account_proxy_address: String,
        name: String,
        description: Option<String>,
        link: Option<String>,
    },
}

/// Callable actions on a remote host
#[cosmwasm_schema::cw_serde]
pub enum HostAction {
    App {
        msg: Binary,
    },
    Dispatch {
        manager_msg: manager::ExecuteMsg,
    },
    SendAllBack {}, // This could be a manager message directly ?
    /// Can't be called through the packet endpoint directly
    Internal(InternalAction),
}

/// Interface to the Host.
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
pub enum ExecuteMsg {
    /// Update the Admin
    UpdateAdmin { admin: String },
    UpdateConfig {
        ans_host_address: Option<String>,
        account_factory_address: Option<String>,
        version_control_address: Option<String>,
    },
    /// Register the Polytone proxy for a specific chain.
    /// proxy should be a local address (will be validated)
    RegisterChainProxy { chain: ChainName, proxy: String },
    /// Remove the Polytone proxy for a specific chain.
    RemoveChainProxy { chain: ChainName },
    /// Allow for fund recovery through the Admin
    RecoverAccount {
        closed_channel: String,
        account_id: AccountId,
        msgs: Vec<CosmosMsg>,
    },
    /// Allows for remote execution from the Polytone implementation
    Execute {
        account_id: AccountId,
        action: HostAction,
    },
}

/// Query Host message
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
pub enum QueryMsg {
    /// Returns [`ConfigResponse`].
    #[returns(ConfigResponse)]
    Config {},
    #[returns(RegisteredChainsResponse)]
    RegisteredChains {},
    #[returns(RegisteredChainResponse)]
    AssociatedClient { chain: String },
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub ans_host_address: Addr,
    pub account_factory_address: Addr,
    pub version_control_address: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct RegisteredChainsResponse {
    pub chains: Vec<(ChainName, String)>,
}

#[cosmwasm_schema::cw_serde]
pub struct RegisteredChainResponse {
    pub proxy: String,
}
