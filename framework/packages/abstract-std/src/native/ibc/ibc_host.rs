use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Binary};

use crate::{
    account::{self, ModuleInstallConfig},
    ibc_client::InstalledModuleIdentification,
    objects::{account::AccountId, module::ModuleInfo, TruncatedChainId},
};

pub mod state {
    use cw_storage_plus::{Item, Map};

    use super::*;
    use crate::objects::storage_namespaces;

    /// Maps a chain name to the proxy it uses to interact on this local chain
    pub const CHAIN_PROXIES: Map<&TruncatedChainId, Addr> =
        Map::new(storage_namespaces::ibc_host::CHAIN_PROXIES);
    pub const REVERSE_CHAIN_PROXIES: Map<&Addr, TruncatedChainId> =
        Map::new(storage_namespaces::ibc_host::REVERSE_CHAIN_PROXIES);

    // Temporary structure to hold actions to be executed after account creation
    pub const TEMP_ACTION_AFTER_CREATION: Item<ActionAfterCreationCache> =
        Item::new(storage_namespaces::ibc_host::TEMP_ACTION_AFTER_CREATION);

    #[cosmwasm_schema::cw_serde]
    pub struct ActionAfterCreationCache {
        pub client_account_address: String,
        pub account_id: AccountId,
        pub action: HostAction,
        pub chain_name: TruncatedChainId,
    }
}
/// Used by Abstract to instantiate the contract
/// The contract is then registered on the registry contract using [`crate::registry::ExecuteMsg::ProposeModules`].
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}

// ANCHOR: ibc-host-action
#[cosmwasm_schema::cw_serde]
#[non_exhaustive]
pub enum InternalAction {
    /// Registers a new account from a remote chain
    Register {
        name: Option<String>,
        description: Option<String>,
        link: Option<String>,
        namespace: Option<String>,
        install_modules: Vec<ModuleInstallConfig>,
    },
}

#[cosmwasm_schema::cw_serde]
#[non_exhaustive]
pub enum HelperAction {
    SendAllBack,
}

/// Callable actions on a remote host
#[cosmwasm_schema::cw_serde]
pub enum HostAction {
    /// Dispatch messages to a remote Account.
    /// Will create a new Account if required.
    Dispatch {
        account_msgs: Vec<account::ExecuteMsg>,
    },
    /// Can't be called by an account directly. These are permissioned messages that only the IBC Client is allowed to call by itself.
    Internal(InternalAction),
    /// Some helpers that allow calling dispatch messages faster (for actions that are called regularly)
    Helpers(HelperAction),
}
// ANCHOR_END: ibc-host-action

/// Interface to the Host.
#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg {
    UpdateOwnership(cw_ownable::Action),
    /// Register the Polytone proxy for a specific chain.
    /// proxy should be a local address (will be validated)
    RegisterChainProxy {
        chain: TruncatedChainId,
        proxy: String,
    },
    /// Remove the Polytone proxy for a specific chain.
    RemoveChainProxy {
        chain: TruncatedChainId,
    },
    // ANCHOR: ibc-host-execute
    /// Allows for remote execution from the Polytone implementation
    #[cw_orch(fn_name("ibc_execute"))]
    Execute {
        account_id: AccountId,
        /// The address of the calling account id. This is used purely for the send-all-back method.
        /// We include it in all messages none-the-less to simplify the users life
        account_address: String,
        action: HostAction,
    },
    // ANCHOR_END: ibc-host-execute
    /// Performs an execution on a local module
    ModuleExecute {
        source_module: InstalledModuleIdentification,
        target_module: ModuleInfo,
        msg: Binary,
    },
    /// Sends the associated funds to the local account corresponding to the source account id
    Fund {
        src_account: AccountId,
        src_chain: TruncatedChainId,
    },
}

/// Query Host message
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
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
    /// Performs an query on a local module
    #[returns(Binary)]
    ModuleQuery {
        target_module: InstalledModuleIdentification,
        msg: Binary,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub ans_host_address: Addr,
    pub module_factory_address: Addr,
    pub registry_address: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct ClientProxiesResponse {
    pub chains: Vec<(TruncatedChainId, Addr)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ClientProxyResponse {
    pub proxy: Addr,
}
