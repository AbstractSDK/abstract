use cosmwasm_std::{instantiate2_address, Binary, CanonicalAddr, Instantiate2AddressError};
use cw_blob::interface::{CwBlob, DeterministicInstantiation};
#[cfg(feature = "daemon")]
use cw_orch::daemon::DeployedChains;

use cw_orch::prelude::*;

use crate::{
    get_ibc_contracts, get_native_contracts, AbstractIbc, AbstractInterfaceError, AccountI,
    AnsHost, ModuleFactory, Registry,
};
use abstract_std::{
    native_addrs, objects::module::ModuleInfo, registry::QueryMsgFns, ACCOUNT, ANS_HOST,
    MODULE_FACTORY, REGISTRY,
};

const CW_BLOB: &str = "cw:blob";

#[derive(Clone)]
pub struct Abstract<Chain: CwEnv> {
    pub ans_host: AnsHost<Chain>,
    pub registry: Registry<Chain>,
    pub module_factory: ModuleFactory<Chain>,
    pub ibc: AbstractIbc<Chain>,
    pub(crate) account: AccountI<Chain>,
    pub(crate) blob: CwBlob<Chain>,
}

impl<Chain: CwEnv> Deploy<Chain> for Abstract<Chain> {
    // We don't have a custom error type
    type Error = AbstractInterfaceError;
    type DeployData = ();

    fn store_on(chain: Chain) -> Result<Self, AbstractInterfaceError> {
        let blob = CwBlob::new(CW_BLOB, chain.clone());

        let ans_host = AnsHost::new(ANS_HOST, chain.clone());
        let registry = Registry::new(REGISTRY, chain.clone());
        let module_factory = ModuleFactory::new(MODULE_FACTORY, chain.clone());
        let account = AccountI::new(ACCOUNT, chain.clone());

        let ibc_infra = AbstractIbc::new(&chain);

        blob.upload_if_needed()?;
        ans_host.upload()?;
        registry.upload()?;
        module_factory.upload()?;
        account.upload()?;
        ibc_infra.upload()?;

        let deployment = Abstract {
            ans_host,
            registry,
            module_factory,
            account,
            ibc: ibc_infra,
            blob,
        };

        Ok(deployment)
    }

    /// Deploys abstract using provided [`TxHandler::Sender`].
    /// After deployment sender of abstract contracts is a sender of provided `chain`
    fn deploy_on(
        chain: Chain,
        _deploy_data: Self::DeployData,
    ) -> Result<Self, AbstractInterfaceError> {
        let sender_addr = chain.sender_addr();
        let admin = sender_addr.to_string();
        // upload
        let deployment = Self::store_on(chain.clone())?;
        let blob_code_id = deployment.blob.code_id()?;

        let creator_account_id: cosmrs::AccountId = admin.as_str().parse().unwrap();
        let canon_creator = CanonicalAddr::from(creator_account_id.to_bytes());

        let expected_addr = |salt: &[u8]| -> Result<CanonicalAddr, Instantiate2AddressError> {
            instantiate2_address(&cw_blob::CHECKSUM, &canon_creator, salt)
        };

        deployment.ans_host.deterministic_instantiate(
            &abstract_std::ans_host::MigrateMsg::Instantiate(
                abstract_std::ans_host::InstantiateMsg {
                    admin: admin.to_string(),
                },
            ),
            blob_code_id,
            expected_addr(native_addrs::ANS_HOST_SALT)?,
            Binary::from(native_addrs::ANS_HOST_SALT),
        )?;

        deployment.registry.deterministic_instantiate(
            &abstract_std::registry::MigrateMsg::Instantiate(
                abstract_std::registry::InstantiateMsg {
                    admin: admin.to_string(),
                    #[cfg(feature = "integration")]
                    security_enabled: Some(false),
                    #[cfg(not(feature = "integration"))]
                    security_enabled: Some(true),
                    namespace_registration_fee: None,
                },
            ),
            blob_code_id,
            expected_addr(native_addrs::REGISTRY_SALT)?,
            Binary::from(native_addrs::REGISTRY_SALT),
        )?;
        deployment.module_factory.deterministic_instantiate(
            &abstract_std::module_factory::MigrateMsg::Instantiate(
                abstract_std::module_factory::InstantiateMsg {
                    admin: admin.to_string(),
                },
            ),
            blob_code_id,
            expected_addr(native_addrs::MODULE_FACTORY_SALT)?,
            Binary::from(native_addrs::MODULE_FACTORY_SALT),
        )?;

        // We also instantiate ibc contracts
        deployment
            .ibc
            .instantiate(&Addr::unchecked(admin.clone()))?;
        deployment.ibc.register(&deployment.registry)?;

        deployment.registry.register_base(&deployment.account)?;
        deployment
            .registry
            .register_natives(deployment.contracts())?;
        deployment.registry.approve_any_abstract_modules()?;

        // Create the first abstract account in integration environments
        #[cfg(feature = "integration")]
        use abstract_std::objects::gov_type::GovernanceDetails;
        #[cfg(feature = "integration")]
        AccountI::create_default_account(
            &deployment,
            GovernanceDetails::Monarchy {
                monarch: chain.sender_addr().to_string(),
            },
        )?;

        Ok(deployment)
    }

    fn get_contracts_mut(&mut self) -> Vec<Box<&mut dyn ContractInstance<Chain>>> {
        vec![
            Box::new(&mut self.ans_host),
            Box::new(&mut self.registry),
            Box::new(&mut self.module_factory),
            Box::new(&mut self.account),
            Box::new(&mut self.ibc.client),
            Box::new(&mut self.ibc.host),
        ]
    }

    fn load_from(chain: Chain) -> Result<Self, Self::Error> {
        #[allow(unused_mut)]
        let mut abstr = Self::new(chain);
        #[cfg(feature = "daemon")]
        {
            // We register all the contracts default state
            let state = crate::AbstractDaemonState::default().state();

            abstr.set_contracts_state(Some(state));
        }
        // Check if abstract deployed, for successful load
        if let Err(CwOrchError::AddrNotInStore(_)) = abstr.registry.address() {
            return Err(AbstractInterfaceError::NotDeployed {});
        } else if abstr.registry.item_query(cw2::CONTRACT).is_err() {
            return Err(AbstractInterfaceError::NotDeployed {});
        }
        Ok(abstr)
    }
}

#[cfg(feature = "daemon")]
impl<Chain: CwEnv> DeployedChains<Chain> for Abstract<Chain> {
    fn deployed_state_file_path() -> Option<String> {
        let crate_path = env!("CARGO_MANIFEST_DIR");

        Some(
            std::path::PathBuf::from(crate_path)
                .join("state.json")
                .display()
                .to_string(),
        )
    }
}

impl<Chain: CwEnv> Abstract<Chain> {
    pub fn new(chain: Chain) -> Self {
        let (ans_host, registry, module_factory) = get_native_contracts(chain.clone());
        let (ibc_client, ibc_host) = get_ibc_contracts(chain.clone());
        let account = AccountI::new(ACCOUNT, chain.clone());
        Self {
            account,
            ans_host,
            registry,
            module_factory,
            ibc: AbstractIbc {
                client: ibc_client,
                host: ibc_host,
            },
            blob: CwBlob::new(CW_BLOB, chain),
        }
    }

    pub fn contracts(&self) -> Vec<(&cw_orch::contract::Contract<Chain>, String)> {
        vec![
            (
                self.ans_host.as_instance(),
                ans_host::contract::CONTRACT_VERSION.to_string(),
            ),
            (
                self.registry.as_instance(),
                registry::contract::CONTRACT_VERSION.to_string(),
            ),
            (
                self.module_factory.as_instance(),
                module_factory::contract::CONTRACT_VERSION.to_string(),
            ),
            (
                self.ibc.client.as_instance(),
                ibc_client::contract::CONTRACT_VERSION.to_string(),
            ),
            (
                self.ibc.host.as_instance(),
                ibc_host::contract::CONTRACT_VERSION.to_string(),
            ),
        ]
    }

    pub fn call_as(&self, sender: &<Chain as TxHandler>::Sender) -> Self {
        Self {
            ans_host: self.ans_host.clone().call_as(sender),
            registry: self.registry.clone().call_as(sender),
            module_factory: self.module_factory.clone().call_as(sender),
            ibc: self.ibc.call_as(sender),
            account: self.account.call_as(sender),
            blob: self.blob.clone(),
        }
    }

    pub fn account_code_id(&self) -> Result<u64, AbstractInterfaceError> {
        let account_module_info = &self
            .registry
            .modules(vec![ModuleInfo::from_id_latest(ACCOUNT)?])?
            .modules[0];

        match account_module_info.module.reference {
            abstract_std::objects::module_reference::ModuleReference::Account(code_id) => Ok(code_id),
            _ => panic!("Your abstract instance has an account module that is not registered as an account. This is bad"),
        }
    }
}
