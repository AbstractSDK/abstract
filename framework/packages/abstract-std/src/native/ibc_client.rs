use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Binary, Coin, Deps, QueryRequest, StdError};
use polytone::callbacks::CallbackMessage;

use self::state::IbcInfrastructure;
use crate::{
    ibc::{Callback, ModuleQuery},
    ibc_host::HostAction,
    manager::{self, ModuleInstallConfig},
    objects::{
        account::AccountId, module::ModuleInfo, module_reference::ModuleReference,
        version_control::VersionControlContract, AssetEntry, TruncatedChainId,
    },
    AbstractError,
};

pub mod state {

    use cosmwasm_std::Addr;
    use cw_storage_plus::{Item, Map};

    use crate::objects::{
        account::{AccountSequence, AccountTrace},
        ans_host::AnsHost,
        version_control::VersionControlContract,
        TruncatedChainId,
    };

    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        pub version_control: VersionControlContract,
        pub ans_host: AnsHost,
    }

    /// Information about the deployed infrastructure we're connected to.
    #[cosmwasm_schema::cw_serde]
    pub struct IbcInfrastructure {
        /// Address of the polytone note deployed on the local chain. This contract will forward the messages for us.
        pub polytone_note: Addr,
        /// The address of the abstract host deployed on the remote chain. This address will be called with our packet.
        pub remote_abstract_host: String,
        // The remote polytone proxy address which will be called by the polytone host.
        pub remote_proxy: Option<String>,
    }

    // Saves the local note deployed contract and the remote abstract host connected
    // This allows sending cross-chain messages
    pub const IBC_INFRA: Map<&TruncatedChainId, IbcInfrastructure> = Map::new("ibci");
    pub const REVERSE_POLYTONE_NOTE: Map<&Addr, TruncatedChainId> = Map::new("revpn");

    pub const CONFIG: Item<Config> = Item::new("config");
    /// (account_trace, account_sequence, chain_name) -> remote proxy account address. We use a
    /// triple instead of including AccountId since nested tuples do not behave as expected due to
    /// a bug that will be fixed in a future release.
    pub const ACCOUNTS: Map<(&AccountTrace, AccountSequence, &TruncatedChainId), String> =
        Map::new("accs");

    // For callbacks tests
    pub const ACKS: Item<Vec<String>> = Item::new("tmpc");
}

/// This needs no info. Owner of the contract is whoever signed the InstantiateMsg.
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub ans_host_address: String,
    pub version_control_address: String,
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}

#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg {
    /// Update the ownership.
    UpdateOwnership(cw_ownable::Action),
    /// Owner method: Registers the polytone note on the local chain as well as the host on the remote chain to send messages through
    /// This allows for monitoring which chain are connected to the contract remotely
    RegisterInfrastructure {
        /// Chain to register the infrastructure for ("juno", "osmosis", etc.)
        chain: TruncatedChainId,
        /// Polytone note (locally deployed)
        note: String,
        /// Address of the abstract host deployed on the remote chain
        host: String,
    },
    /// Owner method: Update the config on IBC client
    UpdateConfig {
        ans_host: Option<String>,
        version_control: Option<String>,
    },
    /// Only callable by Account proxy
    /// Will attempt to forward the specified funds to the corresponding
    /// address on the remote chain.
    SendFunds {
        /// host chain to be executed on
        /// Example: "osmosis"
        host_chain: TruncatedChainId,
        funds: Vec<Coin>,
        memo: Option<String>,
    },
    /// Only callable by Account proxy
    /// Register an Account on a remote chain over IBC
    /// This action creates a proxy for them on the remote chain.
    Register {
        /// host chain to be executed on
        /// Example: "osmosis"
        host_chain: TruncatedChainId,
        base_asset: Option<AssetEntry>,
        namespace: Option<String>,
        install_modules: Vec<ModuleInstallConfig>,
    },
    /// Only callable by Account Module
    // ANCHOR: module-ibc-action
    ModuleIbcAction {
        /// host chain to be executed on
        /// Example: "osmosis"
        host_chain: TruncatedChainId,
        /// Module of this account on host chain
        target_module: ModuleInfo,
        /// Json-encoded IbcMsg to the target module
        msg: Binary,
        /// Callback info to identify the callback that is sent (acts similar to the reply ID)
        callback: Option<Callback>,
    },
    /// Only callable by Account Module
    // ANCHOR_END: module-ibc-action
    IbcQuery {
        /// host chain to be executed on
        /// Example: "osmosis"
        host_chain: TruncatedChainId,
        /// Cosmos Query requests
        queries: Vec<QueryRequest<ModuleQuery>>,
        /// Callback info to identify the callback that is sent (acts similar to the reply ID)
        callback: Callback,
    },
    /// Only callable by Account Proxy
    /// Action on remote ibc host
    /// Which currently only support manager messages
    RemoteAction {
        /// host chain to be executed on
        /// Example: "osmosis"
        host_chain: TruncatedChainId,
        /// execute the custom host function
        action: HostAction,
    },
    /// Owner method: Remove connection for remote chain
    RemoveHost { host_chain: TruncatedChainId },
    /// Callback from the Polytone implementation
    /// This is triggered regardless of the execution result
    Callback(CallbackMessage),
}

/// This enum is used for sending callbacks to the note contract of the IBC client
#[cosmwasm_schema::cw_serde]
pub enum IbcClientCallback {
    ModuleRemoteAction {
        sender_address: String,
        callback: Callback,
        initiator_msg: Binary,
    },
    ModuleRemoteQuery {
        sender_address: String,
        callback: Callback,
        queries: Vec<QueryRequest<ModuleQuery>>,
    },
    CreateAccount {
        account_id: AccountId,
    },
    WhoAmI {},
}

/// This is used for identifying calling modules
/// For adapters, we don't need the account id because it's independent of an account
/// For apps and standalone, the account id is used to identify the calling module
#[cosmwasm_schema::cw_serde]
pub struct InstalledModuleIdentification {
    pub module_info: ModuleInfo,
    pub account_id: Option<AccountId>,
}

#[cosmwasm_schema::cw_serde]
pub struct ModuleAddr {
    pub reference: ModuleReference,
    pub address: Addr,
}

impl InstalledModuleIdentification {
    pub fn addr(
        &self,
        deps: Deps,
        vc: VersionControlContract,
    ) -> Result<ModuleAddr, AbstractError> {
        let target_module_resolved = vc.query_module(self.module_info.clone(), &deps.querier)?;

        let no_account_id_error =
            StdError::generic_err("Account id not specified in installed module definition");

        let target_addr = match &target_module_resolved.reference {
            ModuleReference::AccountBase(code_id) => {
                let target_account_id = self.account_id.clone().ok_or(no_account_id_error)?;
                let account_base = vc.account_base(&target_account_id, &deps.querier)?;

                if deps
                    .querier
                    .query_wasm_contract_info(&account_base.proxy)?
                    .code_id
                    == *code_id
                {
                    account_base.proxy
                } else if deps
                    .querier
                    .query_wasm_contract_info(&account_base.manager)?
                    .code_id
                    == *code_id
                {
                    account_base.manager
                } else {
                    Err(StdError::generic_err(
                        "Account base contract doesn't correspond to any of the proxy or manager",
                    ))?
                }
            }
            ModuleReference::Native(addr) => addr.clone(),
            ModuleReference::Adapter(addr) => addr.clone(),
            ModuleReference::App(_) | ModuleReference::Standalone(_) => {
                let target_account_id = self.account_id.clone().ok_or(no_account_id_error)?;
                let account_base = vc.account_base(&target_account_id, &deps.querier)?;

                let module_info: manager::ModuleAddressesResponse = deps.querier.query_wasm_smart(
                    account_base.manager,
                    &manager::QueryMsg::ModuleAddresses {
                        ids: vec![self.module_info.id()],
                    },
                )?;
                module_info
                    .modules
                    .first()
                    .ok_or(AbstractError::AppNotInstalled(self.module_info.to_string()))?
                    .1
                    .clone()
            }
        };
        Ok(ModuleAddr {
            reference: target_module_resolved.reference,
            address: target_addr,
        })
    }
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum QueryMsg {
    /// Queries the ownership of the ibc client contract
    /// Returns [`cw_ownable::Ownership<Addr>`]
    #[returns(cw_ownable::Ownership<Addr> )]
    Ownership {},

    /// Returns config
    /// Returns [`ConfigResponse`]
    #[returns(ConfigResponse)]
    Config {},

    /// Returns the host information associated with a specific chain-name (e.g. osmosis, juno)
    /// Returns [`HostResponse`]
    #[returns(HostResponse)]
    Host { chain_name: TruncatedChainId },

    /// Get list of remote accounts
    /// Returns [`ListAccountsResponse`]
    #[returns(ListAccountsResponse)]
    ListAccounts {
        start: Option<(AccountId, String)>,
        limit: Option<u32>,
    },

    /// Get remote proxy address for one chain
    /// Returns [`AccountResponse`]
    #[returns(AccountResponse)]
    #[cw_orch(fn_name("remote_account"))]
    Account {
        chain_name: TruncatedChainId,
        account_id: AccountId,
    },

    /// Get the hosts
    /// Returns [`ListRemoteHostsResponse`]
    #[returns(ListRemoteHostsResponse)]
    ListRemoteHosts {},

    /// Get the IBC execution proxies
    /// Returns [`ListRemoteProxiesResponse`]
    #[returns(ListRemoteProxiesResponse)]
    ListRemoteProxies {},

    /// Get the IBC execution proxies based on the account id passed
    /// Returns [`ListRemoteProxiesResponse`]
    #[returns(ListRemoteProxiesResponse)]
    ListRemoteProxiesByAccountId { account_id: AccountId },

    /// Get the IBC counterparts connected to this abstract ibc client
    /// Returns [`ListIbcInfrastructureResponse`]
    #[returns(ListIbcInfrastructureResponse)]
    ListIbcInfrastructures {},
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub ans_host: String,
    pub version_control_address: String,
}

#[cosmwasm_schema::cw_serde]
pub struct ListAccountsResponse {
    pub accounts: Vec<(AccountId, TruncatedChainId, String)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ListRemoteHostsResponse {
    pub hosts: Vec<(TruncatedChainId, String)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ListRemoteProxiesResponse {
    pub proxies: Vec<(TruncatedChainId, Option<String>)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ListIbcInfrastructureResponse {
    pub counterparts: Vec<(TruncatedChainId, IbcInfrastructure)>,
}

#[cosmwasm_schema::cw_serde]
pub struct HostResponse {
    pub remote_host: String,
    pub remote_polytone_proxy: Option<String>,
}

#[cosmwasm_schema::cw_serde]
pub struct AccountResponse {
    pub remote_proxy_addr: Option<String>,
}

#[cosmwasm_schema::cw_serde]
pub struct RemoteProxyResponse {
    /// last block balance was updated (0 is never)
    pub channel_id: String,
    /// address of the remote proxy
    pub proxy_address: String,
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{to_json_binary, CosmosMsg, Empty};
    use speculoos::prelude::*;

    use crate::app::ExecuteMsg;
    use crate::ibc::{Callback, IbcResponseMsg, IbcResult};

    // ... (other test functions)

    #[test]
    fn test_response_msg_to_callback_msg() {
        let receiver = "receiver".to_string();

        let result = IbcResult::FatalError("ibc execution error".to_string());

        let response_msg = IbcResponseMsg {
            callback: Callback::new(&String::from("15")).unwrap(),
            result,
        };

        let actual: CosmosMsg<Empty> = response_msg
            .clone()
            .into_cosmos_msg(receiver.clone())
            .unwrap();

        assert_that!(actual).is_equal_to(CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: receiver,
            // we can't test the message because the fields in it are private
            msg: to_json_binary(&ExecuteMsg::<Empty>::IbcCallback(response_msg)).unwrap(),
            funds: vec![],
        }))
    }
}
