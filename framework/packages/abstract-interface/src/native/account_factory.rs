pub use abstract_std::account_factory::{
    ExecuteMsgFns as AccountFactoryExecFns, QueryMsgFns as AccountFactoryQueryFns,
};
use abstract_std::{
    account_factory::*,
    manager::ModuleInstallConfig,
    objects::{gov_type::GovernanceDetails, AccountId, AssetEntry},
};
use cw_orch::{interface, prelude::*};

use crate::AbstractAccount;

/// A helper struct that contains fields from [`abstract_std::manager::state::AccountInfo`]
#[derive(Default)]
pub struct AccountDetails {
    pub name: String,
    pub description: Option<String>,
    pub link: Option<String>,
    pub namespace: Option<String>,
    pub base_asset: Option<AssetEntry>,
    pub install_modules: Vec<ModuleInstallConfig>,
    pub account_id: Option<u32>,
}

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct AccountFactory<Chain>;

impl<Chain: CwEnv> Uploadable for AccountFactory<Chain> {
    #[cfg(feature = "integration")]
    fn wrapper(&self) -> <Mock as ::cw_orch::environment::TxHandler>::ContractSource {
        Box::new(
            ContractWrapper::new_with_empty(
                ::account_factory::contract::execute,
                ::account_factory::contract::instantiate,
                ::account_factory::contract::query,
            )
            .with_reply_empty(::account_factory::contract::reply)
            .with_migrate(::account_factory::contract::migrate),
        )
    }

    fn wasm(&self) -> WasmPath {
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
        funds: Option<&[Coin]>,
    ) -> Result<AbstractAccount<Chain>, crate::AbstractInterfaceError> {
        let AccountDetails {
            name,
            link,
            description,
            namespace,
            base_asset,
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
                base_asset,
                install_modules,
            },
            funds,
        )?;

        AbstractAccount::from_tx_response(self.get_chain(), result)
    }

    pub fn create_default_account(
        &self,
        governance_details: GovernanceDetails<String>,
    ) -> Result<AbstractAccount<Chain>, crate::AbstractInterfaceError> {
        self.create_new_account(
            AccountDetails {
                name: "Default Abstract Account".into(),
                ..Default::default()
            },
            governance_details,
            None,
        )
    }
}
