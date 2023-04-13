use crate::{
    get_account_contracts, get_native_contracts, AbstractAccount, AbstractBootError,
    AccountFactory, AnsHost, Manager, ModuleFactory, Proxy, VersionControl,
};

use boot_core::*;

pub struct Abstract<Chain: CwEnv> {
    pub ans_host: AnsHost<Chain>,
    pub version_control: VersionControl<Chain>,
    pub account_factory: AccountFactory<Chain>,
    pub module_factory: ModuleFactory<Chain>,
    pub account: AbstractAccount<Chain>,
}

use abstract_core::{ACCOUNT_FACTORY, ANS_HOST, MANAGER, MODULE_FACTORY, PROXY, VERSION_CONTROL};
#[cfg(feature = "integration")]
use boot_core::ContractWrapper;

impl<Chain: CwEnv> boot_core::Deploy<Chain> for Abstract<Chain> {
    // We don't have a custom error type
    type Error = AbstractBootError;
    type DeployData = semver::Version;

    fn store_on(chain: Chain) -> Result<Self, Self::Error> {
        let mut ans_host = AnsHost::new(ANS_HOST, chain.clone());
        let mut account_factory = AccountFactory::new(ACCOUNT_FACTORY, chain.clone());
        let mut version_control = VersionControl::new(VERSION_CONTROL, chain.clone());
        let mut module_factory = ModuleFactory::new(MODULE_FACTORY, chain.clone());
        let mut manager = Manager::new(MANAGER, chain.clone());
        let mut proxy = Proxy::new(PROXY, chain);
        #[cfg(feature = "integration")]
        if cfg!(feature = "integration") {
            ans_host.as_instance_mut().set_mock(Box::new(
                ContractWrapper::new_with_empty(
                    ::ans_host::contract::execute,
                    ::ans_host::contract::instantiate,
                    ::ans_host::contract::query,
                )
                .with_migrate(::ans_host::contract::migrate),
            ));

            account_factory.as_instance_mut().set_mock(Box::new(
                ContractWrapper::new_with_empty(
                    ::account_factory::contract::execute,
                    ::account_factory::contract::instantiate,
                    ::account_factory::contract::query,
                )
                .with_reply_empty(::account_factory::contract::reply)
                .with_migrate(::account_factory::contract::migrate),
            ));

            module_factory.as_instance_mut().set_mock(Box::new(
                boot_core::ContractWrapper::new_with_empty(
                    ::module_factory::contract::execute,
                    ::module_factory::contract::instantiate,
                    ::module_factory::contract::query,
                )
                .with_reply_empty(::module_factory::contract::reply)
                .with_migrate(::module_factory::contract::migrate),
            ));

            version_control.as_instance_mut().set_mock(Box::new(
                boot_core::ContractWrapper::new_with_empty(
                    ::version_control::contract::execute,
                    ::version_control::contract::instantiate,
                    ::version_control::contract::query,
                )
                .with_migrate(::version_control::contract::migrate),
            ));

            manager.as_instance_mut().set_mock(Box::new(
                boot_core::ContractWrapper::new_with_empty(
                    ::manager::contract::execute,
                    ::manager::contract::instantiate,
                    ::manager::contract::query,
                )
                .with_migrate(::manager::contract::migrate),
            ));

            proxy.as_instance_mut().set_mock(Box::new(
                boot_core::ContractWrapper::new_with_empty(
                    ::proxy::contract::execute,
                    ::proxy::contract::instantiate,
                    ::proxy::contract::query,
                )
                .with_migrate(::proxy::contract::migrate),
            ));
        }

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

    fn deploy_on(chain: Chain, version: semver::Version) -> Result<Self, Self::Error> {
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
            .register_base(&deployment.account, &version.to_string())?;

        deployment
            .version_control
            .register_natives(deployment.contracts(), &version)?;
        Ok(deployment)
    }

    fn load_from(chain: Chain) -> Result<Self, Self::Error> {
        Ok(Self::new(chain))
    }
}

impl<Chain: CwEnv> Abstract<Chain> {
    pub fn new(chain: Chain) -> Self {
        let (ans_host, account_factory, version_control, module_factory, _ibc_client) =
            get_native_contracts(chain.clone());
        let (manager, proxy) = get_account_contracts(chain, None);
        Self {
            account: AbstractAccount { manager, proxy },
            ans_host,
            version_control,
            account_factory,
            module_factory,
        }
    }

    pub fn instantiate(&mut self, chain: &Chain) -> Result<(), BootError> {
        let sender = &chain.sender();

        self.ans_host.instantiate(
            &abstract_core::ans_host::InstantiateMsg {},
            Some(sender),
            None,
        )?;

        self.version_control.instantiate(
            &abstract_core::version_control::InstantiateMsg {
                is_testnet: true,
                namespaces_limit: 1,
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

    pub fn contracts(&self) -> Vec<&Contract<Chain>> {
        vec![
            self.ans_host.as_instance(),
            self.version_control.as_instance(),
            self.account_factory.as_instance(),
            self.module_factory.as_instance(),
        ]
    }
}
