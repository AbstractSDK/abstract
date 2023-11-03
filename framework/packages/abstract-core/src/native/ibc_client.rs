use crate::{
    ibc::CallbackInfo,
    ibc_host::HostAction,
    objects::{account::AccountId, chain_name::ChainName},
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Coin, Empty, QueryRequest};
use polytone::callbacks::CallbackMessage;

use self::state::IbcInfrastructure;

pub mod state {

    use crate::objects::{
        account::{AccountSequence, AccountTrace},
        ans_host::AnsHost,
        chain_name::ChainName,
        common_namespace::ADMIN_NAMESPACE,
        version_control::VersionControlContract,
    };
    use cosmwasm_std::Addr;
    use cw_controllers::Admin;
    use cw_storage_plus::{Item, Map};

    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        pub version_control: VersionControlContract,
        pub ans_host: AnsHost,
    }

    pub const ADMIN: Admin = Admin::new(ADMIN_NAMESPACE);

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
    pub const IBC_INFRA: Map<&ChainName, IbcInfrastructure> = Map::new("ibci");
    pub const REVERSE_POLYTONE_NOTE: Map<&Addr, ChainName> = Map::new("revpn");

    pub const CONFIG: Item<Config> = Item::new("config");
    /// (account_trace, account_sequence, chain_name) -> remote proxy account address. We use a
    /// triple instead of including AccountId since nested tuples do not behave as expected due to
    /// a bug that will be fixed in a future release.
    pub const ACCOUNTS: Map<(&AccountTrace, AccountSequence, &ChainName), String> =
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
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
pub enum ExecuteMsg {
    /// Update the Admin
    UpdateAdmin {
        admin: String,
    },
    // Registers the polytone note on the local chain as well as the host on the remote chain to send messages through
    // This allows for monitoring which chain are connected to the contract remotely
    RegisterInfrastructure {
        /// Chain to register the infrastructure for ("juno", "osmosis", etc.)
        chain: String,
        /// Polytone note (locally deployed)
        note: String,
        /// Address of the abstract host deployed on the remote chain
        host: String,
    },
    /// Changes the config
    UpdateConfig {
        ans_host: Option<String>,
        version_control: Option<String>,
    },
    /// Only callable by Account proxy
    /// Will attempt to forward the specified funds to the corresponding
    /// address on the remote chain.
    SendFunds {
        host_chain: String,
        funds: Vec<Coin>,
    },
    /// Register an Account on a remote chain over IBC
    /// This action creates a proxy for them on the remote chain.
    Register {
        host_chain: String,
    },
    RemoteAction {
        // host chain to be executed on
        // Example: "osmosis"
        host_chain: String,
        // execute the custom host function
        action: HostAction,
        // optional callback info
        callback_info: Option<CallbackInfo>,
    },
    /// Allows to query something on a remote contract and act on that query result
    /// This has to be an Execute variant for IBC queries
    RemoteQueries {
        // host chain to be executed on
        // Example: "osmosis"
        host_chain: String,
        // execute following queries
        queries: Vec<QueryRequest<Empty>>,
        // mandatory callback info
        callback_info: CallbackInfo,
    },
    RemoveHost {
        host_chain: String,
    },

    /// Callback from the Polytone implementation
    /// This is only triggered when a contract execution is succesful
    Callback(CallbackMessage),
}

/// This enum is used for sending callbacks to the note contract of the IBC client
#[cosmwasm_schema::cw_serde]
pub enum IbcClientCallback {
    UserRemoteAction(CallbackInfo),
    CreateAccount { account_id: AccountId },
    WhoAmI {},
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // Returns config
    #[returns(HostResponse)]
    Config {},

    // Returns config
    #[returns(HostResponse)]
    Host { chain_name: String },

    // Shows all open channels (incl. remote info)
    #[returns(ListAccountsResponse)]
    ListAccounts {
        start: Option<(AccountId, String)>,
        limit: Option<u32>,
    },

    // Get channel info for one chain
    #[returns(AccountResponse)]
    Account {
        chain: String,
        account_id: AccountId,
    },
    // get the hosts
    #[returns(ListRemoteHostsResponse)]
    ListRemoteHosts {},

    // get the IBC execution proxies
    #[returns(ListRemoteProxiesResponse)]
    ListRemoteProxies {},

    // get the IBC execution proxies based on the account id passed
    #[returns(ListRemoteProxiesResponse)]
    ListRemoteProxiesByAccountId { account_id: AccountId },

    // get the IBC counterparts connected to this abstract client
    #[returns(ListIbcInfrastructureResponse)]
    ListIbcInfrastructures {},
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub ans_host: String,
    pub version_control_address: String,
}

#[cosmwasm_schema::cw_serde]
pub struct ListAccountsResponse {
    pub accounts: Vec<(AccountId, ChainName, String)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ListRemoteHostsResponse {
    pub hosts: Vec<(ChainName, String)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ListRemoteProxiesResponse {
    pub proxies: Vec<(ChainName, Option<String>)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ListIbcInfrastructureResponse {
    pub counterparts: Vec<(ChainName, IbcInfrastructure)>,
}

#[cosmwasm_schema::cw_serde]
pub struct HostResponse {
    pub remote_host: String,
    pub remote_polytone_proxy: Option<String>,
}

#[cosmwasm_schema::cw_serde]
pub struct AccountResponse {
    pub remote_proxy_addr: String,
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
    use crate::ibc::IbcResponseMsg;
    use cosmwasm_std::{to_json_binary, CosmosMsg, Empty};
    use polytone::callbacks::Callback;
    use speculoos::prelude::*;

    // ... (other test functions)

    #[test]
    fn test_response_msg_to_callback_msg() {
        let receiver = "receiver".to_string();
        let callback_id = "15".to_string();
        let callback_msg = to_json_binary("15").unwrap();

        let result = Callback::FatalError("ibc execution error".to_string());

        let response_msg = IbcResponseMsg {
            id: callback_id,
            msg: Some(callback_msg),
            result,
        };

        let actual: CosmosMsg<Empty> = response_msg.into_cosmos_msg(receiver.clone()).unwrap();

        assert_that!(actual).matches(|e| {
            matches!(
                e,
                CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
                    contract_addr: _receiver,
                    // we can't test the message because the fields in it are private
                    msg: _,
                    funds: _
                })
            )
        });
    }
}
