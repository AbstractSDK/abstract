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
    use cosmwasm_std::Addr;
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
    pub const CONTEXT: Item<Context> = Item::new("contxt");
    pub const LOCAL_ACCOUNT_SEQUENCE: Item<AccountSequence> = Item::new("acseq");
}

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Addr;

use crate::{
    manager::ModuleInstallConfig,
    objects::{
        account::{AccountId, AccountSequence, AccountTrace},
        gov_type::GovernanceDetails,
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
    },
    /// Creates the core contracts and sets the permissions.
    /// [`crate::manager`] and [`crate::proxy`]
    #[cw_orch(payable)]
    CreateAccount {
        // Governance details
        governance: GovernanceDetails<String>,
        // Account name
        name: String,
        // Account description
        description: Option<String>,
        // Account link
        link: Option<String>,
        /// Indicates the AccountId for the new account.
        ///
        /// If `None`, will create a new local account without asserting account-id.
        ///
        /// When [`AccountTrace::Local`] when sequence in 0..2147483648 range: The tx will error \
        /// When [`AccountTrace::Local`] when sequence in 2147483648..u32::MAX range: Signals use of unclaimed Account Id in this range. The tx will error if this account-id already claimed. Useful for instantiate2 address prediction. \
        /// When [`AccountTrace::Remote`]: Account id on the remote chain.
        account_id: Option<AccountId>,
        // optionally specify a namespace for the account
        namespace: Option<String>,
        // Provide list of module to install after account creation
        install_modules: Vec<ModuleInstallConfig>,
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
}

/// Account Factory config response
#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub ans_host_contract: Addr,
    pub version_control_contract: Addr,
    pub module_factory_address: Addr,
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
