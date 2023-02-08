use boot_core::{
    prelude::boot_contract, state::StateInterface, BootEnvironment, BootError, Contract,
    IndexResponse, TxResponse,
};
use cosmwasm_std::Addr;

use crate::{Manager, Proxy, OS};
pub use abstract_os::os_factory::{
    ExecuteMsgFns as OsFactoryExecFns, QueryMsgFns as OsFactoryQueryFns,
};
use abstract_os::{objects::gov_type::GovernanceDetails, os_factory::*, ABSTRACT_EVENT_NAME};
use abstract_os::{MANAGER, PROXY};
use boot_core::interface::BootExecute;
use boot_core::interface::ContractInstance;

/// A helper struct that omits fields from [`abstract_os::manager::OsInfo`]
#[derive(Default)]
pub struct OsDetails {
    name: String,
    description: Option<String>,
    link: Option<String>,
}

#[boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct OSFactory<Chain>;

impl<Chain: BootEnvironment> OSFactory<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        let mut contract = Contract::new(name, chain);
        contract = contract.with_wasm_path("os_factory");
        Self(contract)
    }

    pub fn create_new_os(
        &self,
        os_details: OsDetails,
        governance_details: GovernanceDetails,
    ) -> Result<OS<Chain>, BootError> {
        let OsDetails {
            name,
            link,
            description,
        } = os_details;

        let result = self.execute(
            &ExecuteMsg::CreateOs {
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
        Ok(OS {
            manager: Manager::new(MANAGER, self.get_chain().clone()),
            proxy: Proxy::new(PROXY, self.get_chain().clone()),
        })
    }

    pub fn create_default_os(
        &self,
        governance_details: GovernanceDetails,
    ) -> Result<OS<Chain>, BootError> {
        self.create_new_os(
            OsDetails {
                name: "Default Abstract OS".into(),
                ..Default::default()
            },
            governance_details,
        )
    }

    pub fn set_subscription_contract(&self, addr: String) -> Result<TxResponse<Chain>, BootError> {
        self.execute(
            &ExecuteMsg::UpdateConfig {
                admin: None,
                ans_host_contract: None,
                version_control_contract: None,
                module_factory_address: None,
                subscription_address: Some(addr),
            },
            None,
        )
    }
}
