//! # Account Factory
//!
//! `abstract_std::account_factory` handles Account creation and registration.
//!
//! ## Description
//! The Account factory instantiates a new Account instance and registers it with the [`crate::version_control`] contract.  
//! ## Create a new Account
//! Call [`ExecuteMsg::CreateAccount`] on this contract along with a [`crate::objects::gov_type`] and name you'd like to display on your Account.
//!
pub mod state {
    use bs_profile::common::SECONDS_PER_YEAR;
    use cosmwasm_std::{Addr, Uint128};
    use cw_controllers::Admin;
    use cw_storage_plus::Item;
    use serde::{Deserialize, Serialize};

    use crate::{
        objects::{
            account::{AccountId, AccountSequence},
            module::Module,
        },
        version_control::AccountBase,
    };

    /// Account Factory configuration
    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        pub version_control_contract: Addr,
        pub ans_host_contract: Addr,
        pub module_factory_address: Addr,
        pub ibc_host: Option<Addr>,
    }

    /// Tokenized Profile Configuration
    #[cosmwasm_schema::cw_serde]
    pub struct ProfileConfig {
        pub marketplace_addr: Option<String>,
        pub collection_addr: Option<String>,
        pub min_profile_length: u32,
        pub max_profile_length: u32,
        pub max_record_count: u32,
        pub profile_bps: Uint128,
        pub verifier: Option<String>,
    }
    /// Account Factory context for post-[`crate::manager`] [`crate::proxy`] creation
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Context {
        pub account_base: AccountBase,
        pub manager_module: Module,
        pub proxy_module: Module,
        pub account_id: AccountId,
    }

    pub const CONFIG: Item<Config> = Item::new("cfg");
    pub const PROFILE_CONFIG: Item<ProfileConfig> = Item::new("profile-cfg");
    pub const CONTEXT: Item<Context> = Item::new("contxt");
    pub const LOCAL_ACCOUNT_SEQUENCE: Item<AccountSequence> = Item::new("acseq");

    pub const VERIFIER: Admin = Admin::new("verifier");
    pub const IS_PROFILE_SETUP: Item<bool> = Item::new("is-setup");
    /// TODO: remove this and store in registered modules
    pub const PROFILE_MARKETPLACE: Item<Addr> = Item::new("profile-marketplace");
    pub const PROFILE_COLLECTION: Item<Addr> = Item::new("profile-collection");

    pub const TRADING_START_TIME_OFFSET_IN_SECONDS: u64 = 2 * SECONDS_PER_YEAR;
    pub const INIT_PROFIE_COLLECTION_REPLY_ID: u64 = 420;
    pub const INIT_PROFIE_MARKETPLACE_REPLY_ID: u64 = 710;

    // pub const IS_SETUP: Item<bool> = Item::new("is_setup");
}

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Uint128};
use state::ProfileConfig;

use crate::{
    manager::ModuleInstallConfig,
    objects::{
        account::{AccountId, AccountSequence, AccountTrace},
        gov_type::GovernanceDetails,
        AssetEntry,
    },
};

/// Msg used on instantiation
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    /// Admin of the contract
    pub admin: String,
    /// Version control contract used to get code-ids and register Account
    pub version_control_address: String,
    /// AnsHost contract
    pub ans_host_address: String,
    /// AnsHosts of module factory. Used for instantiating manager.
    pub module_factory_address: String,
    pub max_record_count: Option<u32>,
    pub max_profile_length: Option<u32>,
    pub min_profile_length: Option<u32>,
    pub verifier: Option<String>,
    pub profile_bps: Option<Uint128>,
}

/// Account Factory execute messages
#[cw_ownable::cw_ownable_execute]
#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg {
    /// Update config
    UpdateConfig {
        // New ans_host contract
        ans_host_contract: Option<String>,
        // New version control contract
        version_control_contract: Option<String>,
        // New module factory contract
        module_factory_address: Option<String>,
        // New ibc host contract
        ibc_host: Option<String>,
        verifier: Option<String>,
        profile_bps: Option<Uint128>,
        min_profile_length: Option<u32>,
        max_profile_length: Option<u32>,
    },
    /// Creates the core contracts and sets the permissions.
    /// [`crate::manager`] and [`crate::proxy`]
    #[payable]
    CreateAccount {
        // Governance details
        governance: GovernanceDetails<String>,
        // Account name
        name: String,
        // Optionally specify a base asset for the account
        base_asset: Option<AssetEntry>,
        // Account description
        description: Option<String>,
        // Account link
        link: Option<String>,
        /// Indicates the AccountId for the new account.
        ///
        /// If `None`, will create a new local account without asserting account-id.
        ///
        /// When [`AccountTrace::Local`]: Signals the expected Account Id. The tx will error if this does not match the account-id at runtime. Useful for instantiate2 address prediction. \
        /// When [`AccountTrace::Remote`]: Account id on the remote chain.
        account_id: Option<AccountId>,
        // optionally specify a namespace for the account
        namespace: Option<String>,
        // Provide list of module to install after account creation
        install_modules: Vec<ModuleInstallConfig>,

        bs_profile: Option<String>,
    },
    #[cfg_attr(feature = "interface", payable)]
    SetupProfileInfra {
        /// profile collection code id
        profile_code_id: u64,
        /// marketplace code id
        marketplace_code_id: u64,
    },
}

/// Account Factory query messages
#[cw_ownable::cw_ownable_query]
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum QueryMsg {
    /// Returns [`ConfigResponse`]
    #[returns(ConfigResponse)]
    Config {},
    #[returns(ProfileConfig)]
    ProfileConfig {},
}

/// Account Factory config response
#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub ans_host_contract: Addr,
    pub version_control_contract: Addr,
    pub module_factory_address: Addr,
    pub ibc_host: Option<Addr>,
    pub local_account_sequence: AccountSequence,
}

/// Sequence numbers for each origin.
#[cosmwasm_schema::cw_serde]
pub struct SequencesResponse {
    pub sequences: Vec<(AccountTrace, AccountSequence)>,
}

#[cosmwasm_schema::cw_serde]
pub struct SequenceResponse {
    pub sequence: AccountSequence,
}

/// Account Factory migrate messages
#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}

#[cosmwasm_schema::cw_serde]
pub enum SudoMsg {
    UpdateParams { max_record_count: u32 },
}
