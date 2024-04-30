//! # Abstract Api Base
//!
//! `abstract_std::adapter` implements shared functionality that's useful for creating new Abstract adapters.
//!
//! ## Description
//! An Abstract adapter contract is a contract that is allowed to perform actions on a [proxy](crate::proxy) contract.
//! It is not migratable and its functionality is shared between users, meaning that all users call the same contract address to perform operations on the Account.
//! The api structure is well-suited for implementing standard interfaces to external services like dexes, lending platforms, etc.

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Addr;

use crate::{
    manager,
    manager::ModuleInstallConfig,
    objects::{account::AccountId, chain_name::ChainName, AssetEntry},
};

pub mod state {
    use cw_storage_plus::{Item, Map};

    use super::*;
    use crate::objects::{ans_host::AnsHost, version_control::VersionControlContract};

    /// Maps a chain name to the proxy it uses to interact on this local chain
    pub const CHAIN_PROXIES: Map<&ChainName, Addr> = Map::new("ccl");
    pub const REVERSE_CHAIN_PROXIES: Map<&Addr, ChainName> = Map::new("rev-ccl");
    /// Configuration of the IBC host
    pub const CONFIG: Item<Config> = Item::new("cfg");

    // Temporary structure to hold actions to be executed after account creation
    pub const TEMP_ACTION_AFTER_CREATION: Item<ActionAfterCreationCache> = Item::new("act");

    /// The BaseState contains the main addresses needed for sending and verifying messages
    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        /// AnsHost contract struct (address)
        pub ans_host: AnsHost,
        /// Address of the account factory, used to create remote accounts
        pub account_factory: Addr,
        /// Address of the local version control, for retrieving account information
        pub version_control: VersionControlContract,
    }

    #[cosmwasm_schema::cw_serde]
    pub struct ActionAfterCreationCache {
        pub client_proxy_address: String,
        pub account_id: AccountId,
        pub action: HostAction,
        pub chain_name: ChainName,
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
#[non_exhaustive]
pub enum InternalAction {
    /// Registers a new account from a remote chain
    Register {
        name: String,
        description: Option<String>,
        link: Option<String>,
        base_asset: Option<AssetEntry>,
        namespace: Option<String>,
        install_modules: Vec<ModuleInstallConfig>,
    },
}

#[cosmwasm_schema::cw_serde]
#[non_exhaustive]
pub enum HelperAction {
    /// What do we need here ? TODO
    SendAllBack,
}

/// Callable actions on a remote host
#[cosmwasm_schema::cw_serde]
pub enum HostAction {
    Dispatch {
        manager_msg: manager::ExecuteMsg,
    },
    /// Can't be called by an account directly. These are permissioned messages that only the IBC Client is allowed to call by itself.
    Internal(InternalAction),
    /// Some helpers that allow calling dispatch messages faster (for actions that are called regularly)
    Helpers(HelperAction),
}

/// Interface to the Host.
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
pub enum ExecuteMsg {
    UpdateOwnership(cw_ownable::Action),
    UpdateConfig {
        ans_host_address: Option<String>,
        account_factory_address: Option<String>,
        version_control_address: Option<String>,
    },
    /// Register the Polytone proxy for a specific chain.
    /// proxy should be a local address (will be validated)
    RegisterChainProxy {
        chain: String,
        proxy: String,
    },
    /// Remove the Polytone proxy for a specific chain.
    RemoveChainProxy {
        chain: String,
    },
    /// Allows for remote execution from the Polytone implementation
    #[cfg_attr(feature = "interface", fn_name("ibc_execute"))]
    Execute {
        account_id: AccountId,
        /// The address of the calling account id. This is used purely for the send-all-back method.
        /// We include it in all messages one-the-less to simpify the users life
        proxy_address: String,
        action: HostAction,
    },
}

/// Query Host message
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
pub enum QueryMsg {
    /// Queries the ownership of the ibc client contract
    /// Returns [`cw_ownable::Ownership<Addr>`]
    #[returns(cw_ownable::Ownership<Addr> )]
    Ownership {},
    /// Returns [`ConfigResponse`].
    #[returns(ConfigResponse)]
    Config {},
    /// Lists all the polytone proxy contracts and their respective client chain registered with the host.
    /// Returns [`ClientProxiesResponse`].
    #[returns(ClientProxiesResponse)]
    ClientProxies {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Returns the polytone proxy contract address for a specific client chain.
    /// Returns [`ClientProxyResponse`].
    #[returns(ClientProxyResponse)]
    ClientProxy { chain: String },
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub ans_host_address: Addr,
    pub account_factory_address: Addr,
    pub version_control_address: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct ClientProxiesResponse {
    pub chains: Vec<(ChainName, Addr)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ClientProxyResponse {
    pub proxy: Addr,
}
