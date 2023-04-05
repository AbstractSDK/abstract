use crate::{AbstractAccount, Manager, Proxy};
pub use abstract_core::account_factory::{
    ExecuteMsgFns as AccountFactoryExecFns, QueryMsgFns as AccountFactoryQueryFns,
};
use abstract_core::{
    account_factory::*, objects::gov_type::GovernanceDetails, ABSTRACT_EVENT_NAME, MANAGER, PROXY,
};
use boot_core::{
    contract, Contract, CwEnv, IndexResponse, StateInterface, TxResponse,
    {BootExecute, ContractInstance},
};
use cosmwasm_std::Addr;

/// A helper struct that contains fields from [`abstract_core::manager::state::AccountInfo`]
#[derive(Default)]
pub struct AccountDetails {
    name: String,
    description: Option<String>,
    link: Option<String>,
}

#[contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct AccountFactory<Chain>;

impl<Chain: CwEnv> AccountFactory<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        let mut contract = Contract::new(name, chain);
        contract = contract.with_wasm_path("abstract_account_factory");
        Self(contract)
    }

    pub fn create_new_account(
        &self,
        account_details: AccountDetails,
        governance_details: GovernanceDetails,
    ) -> Result<AbstractAccount<Chain>, crate::AbstractBootError> {
        let AccountDetails {
            name,
            link,
            description,
        } = account_details;

        let result = self.execute(
            &ExecuteMsg::CreateAccount {
                governance: governance_details,
                name,
                link,
                description,
            },
            None,
        )?;

        let manager_address = &result.event_attr_value(ABSTRACT_EVENT_NAME, "manager_address")?;
        self.get_chain()
            .state()
            .set_address(MANAGER, &Addr::unchecked(manager_address));
        let proxy_address = &result.event_attr_value(ABSTRACT_EVENT_NAME, "proxy_address")?;
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
        governance_details: GovernanceDetails,
    ) -> Result<AbstractAccount<Chain>, crate::AbstractBootError> {
        self.create_new_account(
            AccountDetails {
                name: "Default Abstract Account".into(),
                ..Default::default()
            },
            governance_details,
        )
    }
}
