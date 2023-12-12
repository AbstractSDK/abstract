use std::path::PathBuf;

use crate::{
    get_ibc_contracts, get_native_contracts, AbstractAccount, AbstractInterfaceError,
    AccountFactory, AnsHost, IbcClient, IbcHost, Manager, ModuleFactory, Proxy, VersionControl,
};
use abstract_core::account_factory::ExecuteMsgFns as _;
use abstract_core::objects::ABSTRACT_ACCOUNT_ID;
use abstract_core::{
    ACCOUNT_FACTORY, ANS_HOST, IBC_CLIENT, IBC_HOST, MANAGER, MODULE_FACTORY, PROXY,
    VERSION_CONTROL,
};
use cw_orch::deploy::Deploy;
use cw_orch::prelude::*;

pub struct IbcAbstract<Chain: CwEnv> {
    pub client: IbcClient<Chain>,
    pub host: IbcHost<Chain>,
}

pub struct Abstract<Chain: CwEnv> {
    pub ans_host: AnsHost<Chain>,
    pub version_control: VersionControl<Chain>,
    pub account_factory: AccountFactory<Chain>,
    pub module_factory: ModuleFactory<Chain>,
    pub ibc: IbcAbstract<Chain>,
    pub(crate) account: AbstractAccount<Chain>,
}

impl<Chain: CwEnv> Deploy<Chain> for Abstract<Chain> {
    // We don't have a custom error type
    type Error = AbstractInterfaceError;
    type DeployData = String;

    fn store_on(chain: Chain) -> Result<Self, AbstractInterfaceError> {
        let ans_host = AnsHost::new(ANS_HOST, chain.clone());
        let account_factory = AccountFactory::new(ACCOUNT_FACTORY, chain.clone());
        let version_control = VersionControl::new(VERSION_CONTROL, chain.clone());
        let module_factory = ModuleFactory::new(MODULE_FACTORY, chain.clone());
        let manager = Manager::new(MANAGER, chain.clone());
        let proxy = Proxy::new(PROXY, chain.clone());

        let ibc_client = IbcClient::new(IBC_CLIENT, chain.clone());
        let ibc_host = IbcHost::new(IBC_HOST, chain.clone());

        let mut account = AbstractAccount { manager, proxy };

        ans_host.upload()?;
        version_control.upload()?;
        account_factory.upload()?;
        module_factory.upload()?;
        account.upload()?;
        ibc_client.upload()?;
        ibc_host.upload()?;

        let deployment = Abstract {
            ans_host,
            account_factory,
            version_control,
            module_factory,
            account,
            ibc: IbcAbstract {
                client: ibc_client,
                host: ibc_host,
            },
        };

        Ok(deployment)
    }

    fn deploy_on(chain: Chain, data: String) -> Result<Self, AbstractInterfaceError> {
        // upload
        let mut deployment = Self::store_on(chain.clone())?;

        // ########### Instantiate ##############
        deployment.instantiate(&chain, data)?;

        // Set Factory
        deployment.version_control.execute(
            &abstract_core::version_control::ExecuteMsg::UpdateConfig {
                account_factory_address: Some(deployment.account_factory.address()?.into_string()),
                namespace_registration_fee: None,
                allow_direct_module_registration_and_updates: None,
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

        // Approve abstract contracts if needed
        deployment.version_control.approve_any_abstract_modules()?;

        // Only the ibc host is allowed to create remote accounts on the account factory
        deployment
            .account_factory
            .update_config(
                None,
                Some(deployment.ibc.host.address().unwrap().to_string()),
                None,
                None,
            )
            .unwrap();

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

    fn deployed_state_file_path() -> Option<String> {
        let crate_path = env!("CARGO_MANIFEST_DIR");

        Some(
            PathBuf::from(crate_path)
                .join("state.json")
                .display()
                .to_string(),
        )
    }

    fn load_from(chain: Chain) -> Result<Self, Self::Error> {
        let mut abstr = Self::new(chain);
        // We register all the contracts default state
        abstr.set_contracts_state(None);
        Ok(abstr)
    }
}

impl<Chain: CwEnv> Abstract<Chain> {
    pub fn new(chain: Chain) -> Self {
        let (ans_host, account_factory, version_control, module_factory) =
            get_native_contracts(chain.clone());
        let (ibc_client, ibc_host) = get_ibc_contracts(chain.clone());
        let manager = Manager::new_from_id(&ABSTRACT_ACCOUNT_ID, chain.clone());
        let proxy = Proxy::new_from_id(&ABSTRACT_ACCOUNT_ID, chain);
        Self {
            account: AbstractAccount { manager, proxy },
            ans_host,
            version_control,
            account_factory,
            module_factory,
            ibc: IbcAbstract {
                client: ibc_client,
                host: ibc_host,
            },
        }
    }

    pub fn instantiate(&mut self, chain: &Chain, admin: String) -> Result<(), CwOrchError> {
        let sender = &chain.sender();

        self.ans_host.instantiate(
            &abstract_core::ans_host::InstantiateMsg {
                admin: admin.clone(),
            },
            Some(sender),
            None,
        )?;

        self.version_control.instantiate(
            &abstract_core::version_control::InstantiateMsg {
                admin: admin.clone(),
                #[cfg(feature = "integration")]
                allow_direct_module_registration_and_updates: Some(true),
                #[cfg(not(feature = "integration"))]
                allow_direct_module_registration_and_updates: Some(false),
                namespace_registration_fee: None,
            },
            Some(sender),
            None,
        )?;

        self.module_factory.instantiate(
            &abstract_core::module_factory::InstantiateMsg {
                admin: admin.clone(),
                version_control_address: self.version_control.address()?.into_string(),
                ans_host_address: self.ans_host.address()?.into_string(),
            },
            Some(sender),
            None,
        )?;

        self.account_factory.instantiate(
            &abstract_core::account_factory::InstantiateMsg {
                admin,
                version_control_address: self.version_control.address()?.into_string(),
                ans_host_address: self.ans_host.address()?.into_string(),
                module_factory_address: self.module_factory.address()?.into_string(),
            },
            Some(sender),
            None,
        )?;

        // We also instantiate ibc contracts
        self.ibc.client.instantiate(
            &abstract_core::ibc_client::InstantiateMsg {
                ans_host_address: self.ans_host.addr_str()?,
                version_control_address: self.version_control.addr_str()?,
            },
            None,
            None,
        )?;

        self.ibc.host.instantiate(
            &abstract_core::ibc_host::InstantiateMsg {
                ans_host_address: self.ans_host.addr_str()?,
                account_factory_address: self.account_factory.addr_str()?,
                version_control_address: self.version_control.addr_str()?,
            },
            None,
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
            (
                self.ibc.client.as_instance(),
                ibc_client::contract::CONTRACT_VERSION.to_string(),
            ),
        ]
    }
}
