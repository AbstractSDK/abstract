use self::state::AccountData;
use crate::{abstract_ica::StdAck, ibc_host::HostAction, objects::account::AccountId};
use abstract_ica::IbcResponseMsg;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{from_slice, Binary, Coin, CosmosMsg, StdResult, Timestamp};

pub mod state {

    use super::LatestQueryResponse;
    use crate::{
        objects::{account::AccountId, ans_host::AnsHost, common_namespace::ADMIN_NAMESPACE},
        ANS_HOST as ANS_HOST_KEY,
    };
    use cosmwasm_std::{Addr, Coin, Timestamp};
    use cw_controllers::Admin;
    use cw_storage_plus::{Item, Map};

    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        pub version_control_address: Addr,
        pub chain: String,
    }

    #[cosmwasm_schema::cw_serde]
    #[derive(Default)]
    pub struct AccountData {
        /// last block balance was updated (0 is never)
        pub last_update_time: Timestamp,
        /// In normal cases, it should be set, but there is a delay between binding
        /// the channel and making a query and in that time it is empty.
        ///
        /// Since we do not have a way to validate the remote address format, this
        /// must not be of type `Addr`.
        pub remote_addr: Option<String>,
        pub remote_balance: Vec<Coin>,
    }

    pub const ADMIN: Admin = Admin::new(ADMIN_NAMESPACE);
    /// host_chain -> channel-id
    pub const CHANNELS: Map<&str, String> = Map::new("channels");
    pub const CONFIG: Item<Config> = Item::new("config");
    /// (channel-id,account_id) -> remote_addr
    pub const ACCOUNTS: Map<(&str, AccountId), AccountData> = Map::new("accounts");
    /// Todo: see if we can remove this
    pub const LATEST_QUERIES: Map<(&str, AccountId), LatestQueryResponse> = Map::new("queries");
    pub const ANS_HOST: Item<AnsHost> = Item::new(ANS_HOST_KEY);
}

/// This needs no info. Owner of the contract is whoever signed the InstantiateMsg.
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub ans_host_address: String,
    pub version_control_address: String,
    pub chain: String,
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}

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
    SendPacket {
        // host chain to be executed on
        // Example: "osmosis"
        host_chain: String,
        // execute the custom host function
        action: HostAction,
        // optional callback info
        callback_info: Option<CallbackInfo>,
        // Number of retries if packet errors
        retries: u8,
    },
    RemoveHost {
        host_chain: String,
    },
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // Returns config
    #[returns(ConfigResponse)]
    Config {},
    // Shows all open channels (incl. remote info)
    #[returns(ListAccountsResponse)]
    ListAccounts {},
    // Get channel info for one chain
    #[returns(AccountResponse)]
    Account {
        chain: String,
        account_id: AccountId,
    },
    // Get remote account info for a chain + Account
    #[returns(LatestQueryResponse)]
    LatestQueryResult {
        chain: String,
        account_id: AccountId,
    },
    // get the channels
    #[returns(ListChannelsResponse)]
    ListChannels {},
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub version_control_address: String,
    pub chain: String,
}

#[cosmwasm_schema::cw_serde]
pub struct ListAccountsResponse {
    pub accounts: Vec<AccountInfo>,
}
#[cosmwasm_schema::cw_serde]
pub struct ListChannelsResponse {
    pub channels: Vec<(String, String)>,
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

#[cosmwasm_schema::cw_serde]
pub struct AccountInfo {
    pub channel_id: String,
    pub account_id: AccountId,
    /// last block balance was updated (0 is never)
    pub last_update_time: Timestamp,
    /// in normal cases, it should be set, but there is a delay between binding
    /// the channel and making a query and in that time it is empty
    pub remote_addr: Option<String>,
    pub remote_balance: Vec<Coin>,
}

impl AccountInfo {
    /// Use the provided *channel_id* and *account_id* to create a new [`AccountInfo`] based off of the provided *input* [`AccountData`].
    pub fn convert(channel_id: String, account_id: AccountId, input: AccountData) -> Self {
        AccountInfo {
            channel_id,
            account_id,
            last_update_time: input.last_update_time,
            remote_addr: input.remote_addr,
            remote_balance: input.remote_balance,
        }
    }
}

#[cosmwasm_schema::cw_serde]
pub struct AccountResponse {
    /// last block balance was updated (0 is never)
    pub last_update_time: Timestamp,
    /// in normal cases, it should be set, but there is a delay between binding
    /// the channel and making a query and in that time it is empty
    pub remote_addr: Option<String>,
    pub remote_balance: Vec<Coin>,
}

impl From<AccountData> for AccountResponse {
    fn from(input: AccountData) -> Self {
        AccountResponse {
            last_update_time: input.last_update_time,
            remote_addr: input.remote_addr,
            remote_balance: input.remote_balance,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::account::TEST_ACCOUNT_ID;
    use cosmwasm_std::to_binary;
    use speculoos::prelude::*;

    // ... (other test functions)

    #[test]
    fn test_account_info_convert() {
        let channel_id = "channel-123".to_string();
        let input = AccountData {
            last_update_time: Default::default(),
            remote_addr: None,
            remote_balance: vec![],
        };

        let expected = AccountInfo {
            channel_id: channel_id.clone(),
            account_id: TEST_ACCOUNT_ID,
            last_update_time: input.last_update_time,
            remote_addr: input.clone().remote_addr,
            remote_balance: input.clone().remote_balance,
        };

        let actual = AccountInfo::convert(channel_id, TEST_ACCOUNT_ID, input);

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn test_account_response_from() {
        let input = AccountData::default();

        let expected = AccountResponse {
            last_update_time: input.last_update_time,
            remote_addr: input.clone().remote_addr,
            remote_balance: input.clone().remote_balance,
        };

        let actual = AccountResponse::from(input);

        assert_that!(actual).is_equal_to(expected);
    }

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
