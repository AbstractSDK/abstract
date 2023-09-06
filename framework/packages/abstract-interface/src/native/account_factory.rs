use crate::{AbstractAccount, Manager, Proxy};
pub use abstract_core::account_factory::{
    ExecuteMsgFns as AccountFactoryExecFns, QueryMsgFns as AccountFactoryQueryFns,
};
use abstract_core::{
    account_factory::*,
    module_factory::ModuleInstallConfig,
    objects::{gov_type::GovernanceDetails, AssetEntry},
    ABSTRACT_EVENT_TYPE, MANAGER, PROXY,
};
use cosmwasm_std::Addr;
use cw_orch::{interface, prelude::*};

/// A helper struct that contains fields from [`abstract_core::manager::state::AccountInfo`]
#[derive(Default)]
pub struct AccountDetails {
    pub name: String,
    pub description: Option<String>,
    pub link: Option<String>,
    pub namespace: Option<String>,
    pub base_asset: Option<AssetEntry>,
    pub install_modules: Vec<ModuleInstallConfig>,
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
            .find_wasm_path("account_factor")
            .unwrap()
    }
}

impl<Chain: CwEnv> AccountFactory<Chain> {
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
        } = account_details;

        let result = self.execute(
            &ExecuteMsg::CreateAccount {
                governance: governance_details,
                name,
                link,
                description,
                namespace,
                base_asset,
                install_modules,
            },
            funds,
        )?;

        let manager_address = &result.event_attr_value(ABSTRACT_EVENT_TYPE, "manager_address")?;
        self.get_chain()
            .state()
            .set_address(MANAGER, &Addr::unchecked(manager_address));
        let proxy_address = &result.event_attr_value(ABSTRACT_EVENT_TYPE, "proxy_address")?;
        self.get_chain()
            .state()
            .set_address(PROXY, &Addr::unchecked(proxy_address));
        Ok(AbstractAccount {
            manager: Manager::new(MANAGER, self.get_chain().clone()),
            proxy: Proxy::new(PROXY, self.get_chain().clone()),
        })
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
