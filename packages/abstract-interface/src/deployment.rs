use std::path::PathBuf;

use crate::{
    get_account_contracts, get_native_contracts, AbstractAccount, AbstractInterfaceError,
    AccountFactory, AnsHost, Manager, ModuleFactory, Proxy, VersionControl,
};
use abstract_core::{ACCOUNT_FACTORY, ANS_HOST, MANAGER, MODULE_FACTORY, PROXY, VERSION_CONTROL};
use cw_orch::deploy::Deploy;
use cw_orch::prelude::*;

pub struct Abstract<Chain: CwEnv> {
    pub ans_host: AnsHost<Chain>,
    pub version_control: VersionControl<Chain>,
    pub account_factory: AccountFactory<Chain>,
    pub module_factory: ModuleFactory<Chain>,
    pub account: AbstractAccount<Chain>,
}

impl<Chain: CwEnv> Deploy<Chain> for Abstract<Chain> {
    // We don't have a custom error type
    type Error = AbstractInterfaceError;
    type DeployData = Empty;

    fn store_on(chain: Chain) -> Result<Self, AbstractInterfaceError> {
        let ans_host = AnsHost::new(ANS_HOST, chain.clone());
        let account_factory = AccountFactory::new(ACCOUNT_FACTORY, chain.clone());
        let version_control = VersionControl::new(VERSION_CONTROL, chain.clone());
        let module_factory = ModuleFactory::new(MODULE_FACTORY, chain.clone());
        let manager = Manager::new(MANAGER, chain.clone());
        let proxy = Proxy::new(PROXY, chain);

        let mut account = AbstractAccount { manager, proxy };

        ans_host.upload()?;
        version_control.upload()?;
        account_factory.upload()?;
        module_factory.upload()?;
        account.upload()?;

        let deployment = Abstract {
            ans_host,
            account_factory,
            version_control,
            module_factory,
            account,
        };

        Ok(deployment)
    }

    fn deploy_on(chain: Chain, _data: Empty) -> Result<Self, AbstractInterfaceError> {
        // upload
        let mut deployment = Self::store_on(chain.clone())?;

        // ########### Instantiate ##############
        deployment.instantiate(&chain)?;

        // Set Factory
        deployment.version_control.execute(
            &abstract_core::version_control::ExecuteMsg::SetFactory {
                new_factory: deployment.account_factory.address()?.into_string(),
            },
            None,
        )?;

        // ########### upload modules and token ##############

        deployment
            .version_control
            .register_base(&deployment.account)?;

        deployment
            .version_control
            .register_natives(deployment.contracts())?;

        // Create the first abstract account in integration environments
        #[cfg(feature = "integration")]
        use abstract_core::objects::gov_type::GovernanceDetails;
        #[cfg(feature = "integration")]
        deployment
            .account_factory
            .create_default_account(GovernanceDetails::Monarchy {
                monarch: chain.sender().to_string(),
            })?;
        Ok(deployment)
    }

    fn get_contracts_mut(&mut self) -> Vec<Box<&mut dyn ContractInstance<Chain>>> {
        vec![
            Box::new(&mut self.ans_host),
            Box::new(&mut self.version_control),
            Box::new(&mut self.account_factory),
            Box::new(&mut self.module_factory),
            Box::new(&mut self.account.manager),
            Box::new(&mut self.account.proxy),
        ]
    }

    fn deployed_state_file_path(&self) -> Option<String> {
        let crate_path = env!("CARGO_MANIFEST_DIR");

        Some(
            PathBuf::from(crate_path)
                .join("daemon_state.json")
                .display()
                .to_string(),
        )
    }

    fn load_from(chain: Chain) -> Result<Self, Self::Error> {
        let mut abstr = Self::new(chain);
        // We register all the contracts default state
        abstr.set_contracts_state();
        Ok(abstr)
    }
}

impl<Chain: CwEnv> Abstract<Chain> {
    pub fn new(chain: Chain) -> Self {
        let (ans_host, account_factory, version_control, module_factory, _ibc_client) =
            get_native_contracts(chain);
        let (manager, proxy) = get_account_contracts(&version_control, None);
        Self {
            account: AbstractAccount { manager, proxy },
            ans_host,
            version_control,
            account_factory,
            module_factory,
        }
    }

    pub fn instantiate(&mut self, chain: &Chain) -> Result<(), CwOrchError> {
        let sender = &chain.sender();

        self.ans_host.instantiate(
            &abstract_core::ans_host::InstantiateMsg {},
            Some(sender),
            None,
        )?;

        self.version_control.instantiate(
            &abstract_core::version_control::InstantiateMsg {
                allow_direct_module_registration: Some(true),
                namespace_limit: 1,
                namespace_registration_fee: None,
            },
            Some(sender),
            None,
        )?;

        self.module_factory.instantiate(
            &abstract_core::module_factory::InstantiateMsg {
                version_control_address: self.version_control.address()?.into_string(),
                ans_host_address: self.ans_host.address()?.into_string(),
            },
            Some(sender),
            None,
        )?;

        self.account_factory.instantiate(
            &abstract_core::account_factory::InstantiateMsg {
                version_control_address: self.version_control.address()?.into_string(),
                ans_host_address: self.ans_host.address()?.into_string(),
                module_factory_address: self.module_factory.address()?.into_string(),
            },
            Some(sender),
            None,
        )?;

        Ok(())
    }

    pub fn contracts(&self) -> Vec<(&cw_orch::contract::Contract<Chain>, String)> {
        vec![
            (
                self.ans_host.as_instance(),
                ans_host::contract::CONTRACT_VERSION.to_string(),
            ),
            (
                self.version_control.as_instance(),
                version_control::contract::CONTRACT_VERSION.to_string(),
            ),
            (
                self.account_factory.as_instance(),
                account_factory::contract::CONTRACT_VERSION.to_string(),
            ),
            (
                self.module_factory.as_instance(),
                module_factory::contract::CONTRACT_VERSION.to_string(),
            ),
        ]
    }
}
