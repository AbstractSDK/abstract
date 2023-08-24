use crate::{
    abstract_ica::StdAck,
    ibc_host::HostAction,
    objects::{account::AccountId, chain_name::ChainName},
};
use abstract_ica::IbcResponseMsg;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{from_slice, Binary, Coin, CosmosMsg, StdResult, Timestamp};
use polytone::callbacks::{CallbackMessage, CallbackRequest};

pub mod state {

    use super::LatestQueryResponse;
    use crate::{
        objects::{
            account::AccountId, ans_host::AnsHost, chain_name::ChainName,
            common_namespace::ADMIN_NAMESPACE,
        },
        ANS_HOST as ANS_HOST_KEY,
    };
    use cosmwasm_std::Addr;
    use cw_controllers::Admin;
    use cw_storage_plus::{Item, Map};

    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        pub version_control_address: Addr,
    }

    pub const ADMIN: Admin = Admin::new(ADMIN_NAMESPACE);

    // Saves the local note deployed contract
    // This allows sending cross-chain messages
    pub const POLYTONE_NOTE: Map<&ChainName, Addr> = Map::new("polytone_note");
    pub const REVERSE_POLYTONE_NOTE: Map<&Addr, ChainName> = Map::new("reverse-polytone_note");
    // Saves the remote ibc host addresses
    // This is used for executing message on the host
    pub const REMOTE_HOST: Map<&ChainName, String> = Map::new("abstract-ibc-hosts");
    pub const REMOTE_PROXY: Map<&ChainName, String> = Map::new("abstract-ibc-client-proxy");

    pub const CONFIG: Item<Config> = Item::new("config");
    /// (account_id, chain_name) -> remote proxy account address
    pub const ACCOUNTS: Map<(&AccountId, &ChainName), String> = Map::new("accounts");
    /// Todo: see if we can remove this
    pub const LATEST_QUERIES: Map<(&str, AccountId), LatestQueryResponse> = Map::new("queries");
    pub const ANS_HOST: Item<AnsHost> = Item::new(ANS_HOST_KEY);

    // For callbacks tests
    pub const ACKS: Item<Vec<String>> = Item::new("temp-callback-storage");
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
pub enum IBCLifecycleComplete {
    #[serde(rename = "ibc_ack")]
    IBCAck {
        /// The source channel (osmosis side) of the IBC packet
        channel: String,
        /// The sequence number that the packet was sent with
        sequence: u64,
        /// String encoded version of the ack as seen by OnAcknowledgementPacket(..)
        ack: String,
        /// Weather an ack is a success of failure according to the transfer spec
        success: bool,
    },
    #[serde(rename = "ibc_timeout")]
    IBCTimeout {
        /// The source channel (osmosis side) of the IBC packet
        channel: String,
        /// The sequence number that the packet was sent with
        sequence: u64,
    },
}

#[cosmwasm_schema::cw_serde]
pub enum SudoMsg {
    #[serde(rename = "ibc_lifecycle_complete")]
    IBCLifecycleComplete(IBCLifecycleComplete),
}

#[cosmwasm_schema::cw_serde]
pub struct CallbackInfo {
    pub id: String,
    pub receiver: String,
}

impl CallbackInfo {
    pub fn to_callback_msg(self, ack_data: &Binary) -> StdResult<CosmosMsg> {
        let msg: StdAck = from_slice(ack_data)?;
        IbcResponseMsg { id: self.id, msg }.into_cosmos_account_msg(self.receiver)
    }
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
pub enum ExecuteMsg {
    /// Update the Admin
    UpdateAdmin {
        admin: String,
    },
    // Registers the polytone note on the local chain as well as the host on the remote chain to send messages through
    // This allows for monitoring which chain are connected to the contract remotely
    RegisterChainHost {
        chain: ChainName,
        note: String,
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
        host_chain: ChainName,
        funds: Vec<Coin>,
    },
    /// Register an Account on a remote chain over IBC
    /// This action creates a proxy for them on the remote chain.
    Register {
        host_chain: ChainName,
    },
    RemoteAction {
        // host chain to be executed on
        // Example: "osmosis"
        host_chain: ChainName,
        // execute the custom host function
        action: HostAction,
        // optional callback info
        callback_request: Option<CallbackRequest>,
    },
    RemoveHost {
        host_chain: ChainName,
    },

    /// Callback from the Polytone implementation
    /// This is only triggered when a contract execution is succesful
    Callback(CallbackMessage),
}

/// This enum is used for sending callbacks to the note contract of the IBC client
#[cosmwasm_schema::cw_serde]
pub enum IbcClientCallback {
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
    Host { chain_name: ChainName },
    // Shows all open channels (incl. remote info)
    #[returns(ListAccountsResponse)]
    ListAccounts {},
    // Get channel info for one chain
    #[returns(AccountResponse)]
    Account {
        chain: String,
        account_id: AccountId,
    },
    // get the hosts
    #[returns(ListRemoteHostsResponse)]
    ListRemoteHosts {},
    // get the IBC execution proxys
    #[returns(ListRemoteProxysResponse)]
    ListRemoteProxys {},
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub version_control_address: String,
    pub chain: String,
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
pub struct ListRemoteProxysResponse {
    pub proxys: Vec<(ChainName, String)>,
}

#[cosmwasm_schema::cw_serde]
pub struct HostResponse {
    pub remote_host: Option<String>,
    pub remote_polytone_proxy: Option<String>,
}
#[cosmwasm_schema::cw_serde]
pub struct AccountResponse {
    pub remote_proxy_addr: String,
}

#[cosmwasm_schema::cw_serde]
pub struct LatestQueryResponse {
    /// last block balance was updated (0 is never)
    pub last_update_time: Timestamp,
    pub response: StdAck,
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
    use super::*;
    use cosmwasm_std::to_binary;
    use speculoos::prelude::*;

    // ... (other test functions)

    #[test]
    fn test_callback_info_to_callback_msg() {
        let receiver = "receiver".to_string();
        let callback_id = "15".to_string();
        let callback_info = CallbackInfo {
            id: callback_id,
            receiver,
        };
        let ack_data = &to_binary(&StdAck::Result(to_binary(&true).unwrap())).unwrap();

        let actual = callback_info.to_callback_msg(&ack_data.clone()).unwrap();

        let _funds: Vec<Coin> = vec![];

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
