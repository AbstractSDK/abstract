use crate::{
    get_apis, get_apps, get_native_contracts, get_os_core_contracts, AnsHost, Manager,
    ModuleFactory, OSFactory, Proxy, VersionControl,
};
use abstract_os::manager::ManagerModuleInfo;
use abstract_os::{
    manager::QueryMsgFns as ManagerQueryMsgFns, proxy::QueryMsgFns as ProxyQueryMsgFns,
};
use boot_core::{prelude::*, BootEnvironment, BootError};
use cosmwasm_std::Empty;
use semver::Version;
use serde::Serialize;
use speculoos::prelude::*;
use std::collections::HashSet;

pub struct Abstract<Chain: BootEnvironment> {
    pub chain: Chain,
    pub version: Version,
    pub ans_host: AnsHost<Chain>,
    pub version_control: VersionControl<Chain>,
    pub os_factory: OSFactory<Chain>,
    pub module_factory: ModuleFactory<Chain>,
}

use abstract_os::{ANS_HOST, MANAGER, MODULE_FACTORY, OS_FACTORY, PROXY, VERSION_CONTROL};

use abstract_os::objects::OsId;
#[cfg(feature = "integration")]
use cw_multi_test::ContractWrapper;

impl<Chain: BootEnvironment> boot_core::deploy::Deploy<Chain> for Abstract<Chain> {
    // We don't have a custom error type
    type Error = BootError;
    type DeployData = semver::Version;

    fn deploy_on(chain: Chain, version: semver::Version) -> Result<Self, BootError> {
        let mut ans_host = AnsHost::new(ANS_HOST, chain.clone());
        let mut os_factory = OSFactory::new(OS_FACTORY, chain.clone());
        let mut version_control = VersionControl::new(VERSION_CONTROL, chain.clone());
        let mut module_factory = ModuleFactory::new(MODULE_FACTORY, chain.clone());
        let mut manager = Manager::new(MANAGER, chain.clone());
        let mut proxy = Proxy::new(PROXY, chain.clone());
        if cfg!(feature = "integration") {
            ans_host
                .as_instance_mut()
                .set_mock(Box::new(ContractWrapper::new_with_empty(
                    ::ans_host::contract::execute,
                    ::ans_host::contract::instantiate,
                    ::ans_host::contract::query,
                )));

            os_factory.as_instance_mut().set_mock(Box::new(
                ContractWrapper::new_with_empty(
                    ::os_factory::contract::execute,
                    ::os_factory::contract::instantiate,
                    ::os_factory::contract::query,
                )
                .with_reply_empty(::os_factory::contract::reply),
            ));

            module_factory.as_instance_mut().set_mock(Box::new(
                cw_multi_test::ContractWrapper::new_with_empty(
                    ::module_factory::contract::execute,
                    ::module_factory::contract::instantiate,
                    ::module_factory::contract::query,
                )
                .with_reply_empty(::module_factory::contract::reply),
            ));

            version_control.as_instance_mut().set_mock(Box::new(
                cw_multi_test::ContractWrapper::new_with_empty(
                    ::version_control::contract::execute,
                    ::version_control::contract::instantiate,
                    ::version_control::contract::query,
                ),
            ));

            manager.as_instance_mut().set_mock(Box::new(
                cw_multi_test::ContractWrapper::new_with_empty(
                    ::manager::contract::execute,
                    ::manager::contract::instantiate,
                    ::manager::contract::query,
                ),
            ));

            proxy.as_instance_mut().set_mock(Box::new(
                cw_multi_test::ContractWrapper::new_with_empty(
                    ::proxy::contract::execute,
                    ::proxy::contract::instantiate,
                    ::proxy::contract::query,
                ),
            ));
        }

        let mut deployment = Abstract {
            chain,
            version,
            ans_host,
            os_factory,
            version_control,
            module_factory,
        };

        let mut os_core = OS { manager, proxy };

        deployment.deploy(&mut os_core)?;
        Ok(deployment)
    }

    fn load_from(chain: Chain) -> Result<Self, Self::Error> {
        let (ans_host, os_factory, version_control, module_factory, _ibc_client) =
            get_native_contracts(chain.clone());
        let version = env!("CARGO_PKG_VERSION").parse().unwrap();
        Ok(Self {
            chain,
            version,
            ans_host,
            version_control,
            os_factory,
            module_factory,
        })
    }
}

impl<Chain: BootEnvironment> Abstract<Chain> {
    pub fn new(chain: Chain, version: Version) -> Self {
        let (ans_host, os_factory, version_control, module_factory, _ibc_client) =
            get_native_contracts(chain.clone());

        Self {
            chain,
            ans_host,
            version_control,
            os_factory,
            module_factory,
            version,
        }
    }

    fn get_chain(&self) -> Chain {
        self.chain.clone()
    }

    pub fn deploy(&mut self, os_core: &mut OS<Chain>) -> Result<(), BootError> {
        let sender = &self.chain.sender();

        // ########### Upload ##############

        self.ans_host.upload()?;
        self.version_control.upload()?;
        self.os_factory.upload()?;
        self.module_factory.upload()?;

        os_core.upload()?;

        // ########### Instantiate ##############

        self.ans_host.instantiate(
            &abstract_os::ans_host::InstantiateMsg {},
            Some(sender),
            None,
        )?;

        self.version_control.instantiate(
            &abstract_os::version_control::InstantiateMsg {},
            Some(sender),
            None,
        )?;

        self.module_factory.instantiate(
            &abstract_os::module_factory::InstantiateMsg {
                version_control_address: self.version_control.address()?.into_string(),
                ans_host_address: self.ans_host.address()?.into_string(),
            },
            Some(sender),
            None,
        )?;

        self.os_factory.instantiate(
            &abstract_os::os_factory::InstantiateMsg {
                version_control_address: self.version_control.address()?.into_string(),
                ans_host_address: self.ans_host.address()?.into_string(),
                module_factory_address: self.module_factory.address()?.into_string(),
            },
            Some(sender),
            None,
        )?;

        // Set Factory
        self.version_control.execute(
            &abstract_os::version_control::ExecuteMsg::SetFactory {
                new_factory: self.os_factory.address()?.into_string(),
            },
            None,
        )?;

        // ########### upload modules and token ##############

        self.version_control
            .register_core(os_core, &self.version.to_string())?;

        self.version_control.register_deployment(self)?;

        Ok(())
    }

    pub fn deploy_modules(&self) -> Result<(), BootError> {
        self.upload_modules()?;
        self.instantiate_apis()?;
        self.register_modules()?;
        Ok(())
    }

    pub fn contracts(&self) -> Vec<&Contract<Chain>> {
        vec![
            self.ans_host.as_instance(),
            self.version_control.as_instance(),
            self.os_factory.as_instance(),
            self.module_factory.as_instance(),
        ]
    }

    fn instantiate_apis(&self) -> Result<(), BootError> {
        let (dex, staking) = get_apis(self.get_chain());
        let init_msg = abstract_os::api::InstantiateMsg {
            app: Empty {},
            base: abstract_os::api::BaseInstantiateMsg {
                ans_host_address: self.ans_host.address()?.into(),
                version_control_address: self.version_control.address()?.into(),
            },
        };
        dex.instantiate(&init_msg, None, None)?;
        staking.instantiate(&init_msg, None, None)?;
        Ok(())
    }

    fn upload_modules(&self) -> Result<(), BootError> {
        let (mut dex, mut staking) = get_apis(self.get_chain());
        let (mut etf, mut subs) = get_apps(self.get_chain());
        let modules: Vec<&mut dyn BootUpload<Chain>> =
            vec![&mut dex, &mut staking, &mut etf, &mut subs];
        modules
            .into_iter()
            .map(BootUpload::upload)
            .collect::<Result<Vec<_>, BootError>>()?;
        Ok(())
    }

    fn register_modules(&self) -> Result<(), BootError> {
        let (dex, staking) = get_apis(self.get_chain());
        let (etf, subs) = get_apps(self.get_chain());

        self.version_control
            .register_apps(vec![etf.as_instance(), subs.as_instance()], &self.version)?;
        self.version_control.register_apis(
            vec![dex.as_instance(), staking.as_instance()],
            &self.version,
        )?;
        Ok(())
    }
}

pub struct OS<Chain: BootEnvironment> {
    pub manager: Manager<Chain>,
    pub proxy: Proxy<Chain>,
}

impl<Chain: BootEnvironment> OS<Chain> {
    pub fn new(chain: Chain, os_id: Option<OsId>) -> Self {
        let (manager, proxy) = get_os_core_contracts(chain, os_id);
        Self { manager, proxy }
    }

    pub fn upload(&mut self) -> Result<(), BootError> {
        self.manager.upload()?;
        self.proxy.upload()?;
        Ok(())
    }

    /// Register the os core contracts in the version control
    pub fn register(
        &self,
        version_control: &VersionControl<Chain>,
        version: &str,
    ) -> Result<(), BootError> {
        version_control.register_core(self, version)
    }

    pub fn install_module<TInitMsg: Serialize>(
        &mut self,
        module_id: &str,
        init_msg: &TInitMsg,
    ) -> Result<(), BootError> {
        self.manager.install_module(module_id, init_msg)
    }

    /// Assert that the OS has the expected modules with the provided **expected_module_addrs** installed.
    /// Also checks that the proxy's configuration includes the expected module addresses.
    /// Note that the proxy is automatically included in the assertions and *should not* (but can) be included in the expected list.
    /// Returns the Vec<ManagerModuleInfo> from the manager
    pub fn expect_modules(
        &self,
        module_addrs: Vec<String>,
    ) -> Result<Vec<ManagerModuleInfo>, BootError> {
        let abstract_os::manager::ModuleInfosResponse {
            module_infos: manager_modules,
        } = self.manager.module_infos(None, None)?;

        let expected_module_addrs = module_addrs
            .into_iter()
            .chain(std::iter::once(self.manager.address()?.into_string()))
            .collect::<HashSet<_>>();

        // account for the proxy
        assert_that!(manager_modules).has_length(expected_module_addrs.len());

        // check proxy config
        let abstract_os::proxy::ConfigResponse {
            modules: proxy_whitelist,
        } = self.proxy.config()?;

        let actual_proxy_whitelist = HashSet::from_iter(proxy_whitelist);
        assert_eq!(actual_proxy_whitelist, expected_module_addrs);

        Ok(manager_modules)
    }
}
