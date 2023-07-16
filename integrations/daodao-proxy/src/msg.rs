//! # Dao Account Proxy

use crate::{
    ibc_client::ExecuteMsg as IbcClientMsg,
    objects::{
        account_id::AccountId,
        oracle::{AccountValue, Complexity},
        price_source::{PriceSource, UncheckedPriceSource},
        AssetEntry,
    },
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{CosmosMsg, Empty, Uint128};
use cw_asset::{Asset, AssetInfo};

pub mod state {
    pub use crate::objects::account_id::ACCOUNT_ID;
    use cw_controllers::Admin;

    use cosmwasm_std::Addr;
    use cw_storage_plus::Item;

    use crate::objects::{ans_host::AnsHost, common_namespace::ADMIN_NAMESPACE};
    #[cosmwasm_schema::cw_serde]
    pub struct State {
        pub modules: Vec<Addr>,
    }
    pub const ANS_HOST: Item<AnsHost> = Item::new("\u{0}{6}ans_host");
    pub const STATE: Item<State> = Item::new("\u{0}{5}state");
    pub const ADMIN: Admin = Admin::new(ADMIN_NAMESPACE);
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {

}


#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
pub enum ExecuteMsg {
    
    // # Abstract Messages
    /// Sets the admin
    SetAdmin { admin: String },
    /// Executes the provided messages if sender is whitelisted
    ModuleAction { msgs: Vec<CosmosMsg<Empty>> },
    /// Execute a message and forward the Response data
    ModuleActionWithData { msg: CosmosMsg<Empty> },
    /// Execute IBC action on Client
    IbcAction { msgs: Vec<IbcClientMsg> },
    /// Adds the provided address to whitelisted dapps
    AddModule { module: String },
    /// Removes the provided address from the whitelisted dapps
    RemoveModule { module: String },
    /// Updates the VAULT_ASSETS map
    UpdateAssets {
        to_add: Vec<(AssetEntry, UncheckedPriceSource)>,
        to_remove: Vec<AssetEntry>,
    },
    
    // # DAO-DAO Messages
    /// Callable by the Admin, if one is configured.
    /// Executes messages in order.
    ExecuteAdminMsgs { msgs: Vec<CosmosMsg<Empty>> },
    /// Callable by proposal modules. The DAO will execute the
    /// messages in the hook in order.
    ExecuteProposalHook { msgs: Vec<CosmosMsg<Empty>> },
    /// Pauses the DAO for a set duration.
    /// When paused the DAO is unable to execute proposals
    Pause { duration: Duration },
    /// Executed when the contract receives a cw20 token. Depending on
    /// the contract's configuration the contract will automatically
    /// add the token to its treasury.
    Receive(cw20::Cw20ReceiveMsg),
    /// Executed when the contract receives a cw721 token. Depending
    /// on the contract's configuration the contract will
    /// automatically add the token to its treasury.
    ReceiveNft(cw721::Cw721ReceiveMsg),
    /// Removes an item from the governance contract's item map.
    RemoveItem { key: String },
    /// Adds an item to the governance contract's item map. If the
    /// item already exists the existing value is overridden. If the
    /// item does not exist a new item is added.
    SetItem { key: String, value: String },
    /// Callable by the admin of the contract. If ADMIN is None the
    /// admin is set as the contract itself so that it may be updated
    /// later by vote. If ADMIN is Some a new admin is proposed and
    /// that new admin may become the admin by executing the
    /// `AcceptAdminNomination` message.
    ///
    /// If there is already a pending admin nomination the
    /// `WithdrawAdminNomination` message must be executed before a
    /// new admin may be nominated.
    NominateAdmin { admin: Option<String> },
    /// Callable by a nominated admin. Admins are nominated via the
    /// `NominateAdmin` message. Accepting a nomination will make the
    /// nominated address the new admin.
    ///
    /// Requiring that the new admin accepts the nomination before
    /// becoming the admin protects against a typo causing the admin
    /// to change to an invalid address.
    AcceptAdminNomination {},
    /// Callable by the current admin. Withdraws the current admin
    /// nomination.
    WithdrawAdminNomination {},
    /// Callable by the core contract. Replaces the current
    /// governance contract config with the provided config.
    UpdateConfig { config: Config },
    /// Updates the list of cw20 tokens this contract has registered.
    UpdateCw20List {
        to_add: Vec<String>,
        to_remove: Vec<String>,
    },
    /// Updates the list of cw721 tokens this contract has registered.
    UpdateCw721List {
        to_add: Vec<String>,
        to_remove: Vec<String>,
    },
    /// Updates the governance contract's governance modules. Module
    /// instantiate info in `to_add` is used to create new modules and
    /// install them.
    UpdateProposalModules {
        /// NOTE: the pre-propose-base package depends on it being the
        /// case that the core module instantiates its proposal module.
        to_add: Vec<ModuleInstantiateInfo>,
        to_disable: Vec<String>,
    },
    /// Callable by the core contract. Replaces the current
    /// voting module with a new one instantiated by the governance
    /// contract.
    UpdateVotingModule { module: ModuleInstantiateInfo },
    /// Update the core module to add/remove SubDAOs and their charters
    UpdateSubDaos {
        to_add: Vec<SubDao>,
        to_remove: Vec<String>,
    },
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
pub enum QueryMsg {
    /// Contains the enabled modules
    /// Returns [`ConfigResponse`]
    #[returns(abstract_core::proxy::ConfigResponse)]
    Config {},
    /// Returns the total value of the assets held by this account
    /// [`AccountValue`]
    #[returns(abstract_core::objects::oracle::AccountValue)]
    TotalValue {},
    /// Returns the value of one token with an optional amount set.
    /// If amount is not set, the account's balance of the token is used.
    /// [`TokenValueResponse`]
    #[returns(abstract_core::proxy::TokenValueResponse)]
    TokenValue { identifier: AssetEntry },
    /// Returns the amount of specified tokens this contract holds
    /// [`HoldingAmountResponse`]
    #[returns(abstract_core::proxy::HoldingAmountResponse)]
    HoldingAmount { identifier: AssetEntry },
    /// Returns the oracle configuration value for the specified key
    /// [`AssetConfigResponse`]
    #[returns(abstract_core::proxy::AssetConfigResponse)]
    AssetConfig { identifier: AssetEntry },
    /// Returns [`AssetsConfigResponse`]
    /// Human readable
    #[returns(abstract_core::proxy::AssetsConfigResponse)]
    AssetsConfig {
        start_after: Option<AssetEntry>,
        limit: Option<u8>,
    },
    /// Returns [`AssetsInfoResponse`]
    /// Not human readable
    #[returns(abstract_core::proxy::AssetsInfoResponse)]
    AssetsInfo {
        start_after: Option<AssetInfo>,
        limit: Option<u8>,
    },
    /// Returns [`BaseAssetResponse`]
    #[returns(abstract_core::proxy::BaseAssetResponse)]
    BaseAsset {},
}