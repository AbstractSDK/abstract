use cosmwasm_std::{Binary, CanonicalAddr};
use cw_blob::interface::{CwBlob, DeterministicInstantiation};
#[cfg(feature = "daemon")]
use cw_orch::daemon::DeployedChains;

use cw_orch::{mock::MockBase, prelude::*};

use crate::{
    get_ibc_contracts, get_native_contracts, AbstractIbc, AbstractInterfaceError, AccountI,
    AnsHost, ModuleFactory, Registry,
};
use abstract_std::{
    native_addrs, ACCOUNT, ANS_HOST, IBC_CLIENT, IBC_HOST, MODULE_FACTORY, REGISTRY,
};

use rust_embed::RustEmbed;

const CW_BLOB: &str = "cw:blob";

#[derive(RustEmbed)]
// Can't use symlinks in debug mode
// https://github.com/pyrossh/rust-embed/pull/234
#[folder = "./"]
#[include = "state.json"]
struct State;

impl State {
    #[allow(unused)]
    pub fn load_state() -> serde_json::Value {
        let state_file =
            State::get("state.json").expect("Unable to read abstract-interface state.json");
        serde_json::from_slice(&state_file.data).unwrap()
    }
}

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
    type DeployData = Chain::Sender;

    fn store_on(chain: Chain) -> Result<Self, AbstractInterfaceError> {
        let blob = CwBlob::new(CW_BLOB, chain.clone());

        let ans_host = AnsHost::new(ANS_HOST, chain.clone());
        let registry = Registry::new(REGISTRY, chain.clone());
        let module_factory = ModuleFactory::new(MODULE_FACTORY, chain.clone());
        let account = AccountI::new(ACCOUNT, chain.clone());

        let ibc_infra = AbstractIbc::new(&chain);

        blob.upload()?;
        ans_host.upload()?;
        registry.upload()?;
        module_factory.upload()?;
        account.upload()?;
        ibc_infra.upload()?;

        let deployment = Abstract {
            ans_host,
            registry: registry,
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
        mut chain: Chain,
        deploy_data: Self::DeployData,
    ) -> Result<Self, AbstractInterfaceError> {
        let original_sender = chain.sender().clone();
        chain.set_sender(deploy_data);
        let admin = chain.sender_addr().to_string();
        // upload
        let mut deployment = Self::store_on(chain.clone())?;
        let blob_code_id = deployment.blob.code_id()?;

        deployment.ans_host.deterministic_instantiate(
            &abstract_std::ans_host::MigrateMsg::Instantiate(
                abstract_std::ans_host::InstantiateMsg {
                    admin: admin.to_string(),
                },
            ),
            blob_code_id,
            CanonicalAddr::from(native_addrs::ANS_ADDR),
            Binary::from(ANS_HOST.as_bytes()),
        )?;

        deployment.registry.deterministic_instantiate(
            &abstract_std::registry::MigrateMsg::Instantiate(
                abstract_std::registry::InstantiateMsg {
                    admin: admin.to_string(),
                    #[cfg(feature = "integration")]
                    security_disabled: Some(true),
                    #[cfg(not(feature = "integration"))]
                    security_disabled: Some(false),
                    namespace_registration_fee: None,
                },
            ),
            blob_code_id,
            CanonicalAddr::from(native_addrs::REGISTRY_ADDR),
            Binary::from(REGISTRY.as_bytes()),
        )?;
        deployment.module_factory.deterministic_instantiate(
            &abstract_std::module_factory::MigrateMsg::Instantiate(
                abstract_std::module_factory::InstantiateMsg {
                    admin: admin.to_string(),
                },
            ),
            blob_code_id,
            CanonicalAddr::from(native_addrs::MODULE_FACTORY_ADDR),
            Binary::from(MODULE_FACTORY.as_bytes()),
        )?;

        // We also instantiate ibc contracts
        deployment.ibc.client.deterministic_instantiate(
            &abstract_std::ibc_client::MigrateMsg::Instantiate(
                abstract_std::ibc_client::InstantiateMsg {
                    ans_host_address: deployment.ans_host.addr_str()?,
                    registry_address: deployment.registry.addr_str()?,
                },
            ),
            blob_code_id,
            CanonicalAddr::from(native_addrs::IBC_CLIENT_ADDR),
            Binary::from(IBC_CLIENT.as_bytes()),
        )?;
        deployment.ibc.host.deterministic_instantiate(
            &abstract_std::ibc_host::MigrateMsg::Instantiate(
                abstract_std::ibc_host::InstantiateMsg {},
            ),
            blob_code_id,
            CanonicalAddr::from(native_addrs::IBC_HOST_ADDR),
            Binary::from(IBC_HOST.as_bytes()),
        )?;
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

        // Return original sender
        deployment.update_sender(&original_sender);
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
            let state = State::load_state();

            abstr.set_contracts_state(Some(state));
        }
        // Check if abstract deployed, for successful load
        if let Err(CwOrchError::AddrNotInStore(_)) = abstr.registry.address() {
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
            registry: registry,
            module_factory,
            ibc: AbstractIbc {
                client: ibc_client,
                host: ibc_host,
            },
            blob: CwBlob::new(CW_BLOB, chain),
        }
    }

    pub fn instantiate(&mut self, admin: String) -> Result<(), AbstractInterfaceError> {
        let admin = Addr::unchecked(admin);

        self.ans_host.instantiate(
            &abstract_std::ans_host::InstantiateMsg {
                admin: admin.to_string(),
            },
            Some(&admin),
            &[],
        )?;

        self.registry.instantiate(
            &abstract_std::registry::InstantiateMsg {
                admin: admin.to_string(),
                #[cfg(feature = "integration")]
                security_disabled: Some(true),
                #[cfg(not(feature = "integration"))]
                security_disabled: Some(false),
                namespace_registration_fee: None,
            },
            Some(&admin),
            &[],
        )?;

        self.module_factory.instantiate(
            &abstract_std::module_factory::InstantiateMsg {
                admin: admin.to_string(),
            },
            Some(&admin),
            &[],
        )?;

        // We also instantiate ibc contracts
        self.ibc.instantiate(self, &admin)?;
        self.ibc.register(&self.registry)?;

        Ok(())
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

    pub fn update_sender(&mut self, sender: &Chain::Sender) {
        let Self {
            ans_host,
            registry: registry,
            module_factory,
            ibc,
            account,
            blob: _,
        } = self;
        ans_host.set_sender(sender);
        registry.set_sender(sender);
        module_factory.set_sender(sender);
        account.set_sender(sender);
        ibc.client.set_sender(sender);
        ibc.host.set_sender(sender);
    }
}

// Sender addr means it's mock or CloneTest(which is also mock)
impl<Chain: CwEnv<Sender = Addr>> Abstract<Chain> {
    pub fn deploy_on_mock(chain: Chain) -> Result<Self, AbstractInterfaceError> {
        let admin = Self::mock_admin(&chain);
        Self::deploy_on(chain, admin)
    }

    pub fn mock_admin(chain: &Chain) -> <MockBase as TxHandler>::Sender {
        // Getting prefix
        let sender_addr: cosmrs::AccountId = chain.sender().as_str().parse().unwrap();
        let prefix = sender_addr.prefix();
        // Building mock_admin
        let mock_admin =
            cosmrs::AccountId::new(prefix, &native_addrs::TEST_ABSTRACT_CREATOR).unwrap();
        Addr::unchecked(mock_admin)
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use std::borrow::Cow;

    use cosmwasm_std::Api;
    use cw_orch::anyhow;
    use native_addrs::TEST_ABSTRACT_CREATOR;

    use super::*;

    #[test]
    fn only_state_json_included() {
        let files = State::iter().collect::<Vec<_>>();
        assert_eq!(files, vec![Cow::Borrowed("state.json")])
    }

    #[test]
    fn have_some_state() {
        State::get("state.json").unwrap();
        let state = State::load_state();
        // TODO: remove ["juno"] after updating state, we only need chain_id now
        let vc_juno = &state["juno"]["juno-1"]["code_ids"].get(REGISTRY);
        assert!(vc_juno.is_some());
    }

    #[test]
    fn deploy2() -> anyhow::Result<()> {
        let mut chain = MockBech32::new("mock");
        let sender = chain
            .app
            .borrow()
            .api()
            .addr_humanize(&CanonicalAddr::from(TEST_ABSTRACT_CREATOR))?;
        chain.set_sender(sender);

        let abstr = Abstract::deploy_on(chain.clone(), chain.sender().clone())?;
        let app = chain.app.borrow();
        let api = app.api();

        // ANS
        let ans_addr = api.addr_canonicalize(&abstr.ans_host.addr_str()?)?;
        assert_eq!(*ans_addr, native_addrs::ANS_ADDR);

        // VC
        let registry = api.addr_canonicalize(&abstr.registry.addr_str()?)?;
        assert_eq!(*registry, native_addrs::REGISTRY_ADDR);

        // MODULE_FACTORY
        let module_factory = api.addr_canonicalize(&abstr.module_factory.addr_str()?)?;
        assert_eq!(*module_factory, native_addrs::MODULE_FACTORY_ADDR);

        // IBC_CLIENT
        let ibc_client = api.addr_canonicalize(&abstr.ibc.client.addr_str()?)?;
        assert_eq!(*ibc_client, native_addrs::IBC_CLIENT_ADDR);

        // IBC_HOST
        let ibc_host = api.addr_canonicalize(&abstr.ibc.host.addr_str()?)?;
        assert_eq!(*ibc_host, native_addrs::IBC_HOST_ADDR);

        Ok(())
    }
}
