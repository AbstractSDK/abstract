pub use abstract_std::account_factory::{
    ExecuteMsgFns as AccountFactoryExecFns, QueryMsgFns as AccountFactoryQueryFns,
};
use abstract_std::{
    account::ModuleInstallConfig,
    account_factory::*,
    objects::{gov_type::GovernanceDetails, AccountId},
};
use cw_orch::{environment::Environment, interface, prelude::*};

use crate::AccountI;

/// A helper struct that contains fields from [`abstract_std::manager::state::AccountInfo`]
#[derive(Default)]
pub struct AccountDetails {
    pub name: String,
    pub description: Option<String>,
    pub link: Option<String>,
    pub namespace: Option<String>,
    pub install_modules: Vec<ModuleInstallConfig>,
    pub account_id: Option<u32>,
}

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct AccountFactory<Chain>;

impl<Chain: CwEnv> Uploadable for AccountFactory<Chain> {
    #[cfg(feature = "integration")]
    fn wrapper() -> <Mock as ::cw_orch::environment::TxHandler>::ContractSource {
        Box::new(
            ContractWrapper::new_with_empty(
                ::account_factory::contract::execute,
                ::account_factory::contract::instantiate,
                ::account_factory::contract::query,
            )
            .with_reply_empty(::account_factory::contract::reply)
            .with_migrate(::account_factory::migrate::migrate),
        )
    }

    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("account_factory")
            .unwrap()
    }
}

impl<Chain: CwEnv> AccountFactory<Chain> {
    /// Creates a local account
    pub fn create_new_account(
        &self,
        account_details: AccountDetails,
        governance_details: GovernanceDetails<String>,
        funds: &[Coin],
    ) -> Result<AccountI<Chain>, crate::AbstractInterfaceError> {
        let AccountDetails {
            name,
            link,
            description,
            namespace,
            install_modules,
            account_id,
        } = account_details;

        let result = self.execute(
            &ExecuteMsg::CreateAccount {
                governance: governance_details,
                name,
                link,
                description,
                account_id: account_id.map(AccountId::local),
                namespace,
                install_modules,
            },
            funds,
        )?;

        AccountI::from_tx_response(self.environment(), result)
    }

    pub fn create_default_account(
        &self,
        governance_details: GovernanceDetails<String>,
    ) -> Result<AccountI<Chain>, crate::AbstractInterfaceError> {
        self.create_new_account(
            AccountDetails {
                name: "Default Abstract Account".into(),
                ..Default::default()
            },
            governance_details,
            &[],
        )
    }
}
