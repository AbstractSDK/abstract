use cosmwasm_std::{instantiate2_address, Binary, CanonicalAddr};
use cw_blob::interface::CwBlob;
#[cfg(feature = "daemon")]
use cw_orch::daemon::DeployedChains;

use cw_orch::{contract::Contract, prelude::*};

use crate::{
    get_ibc_contracts, get_native_contracts, AbstractAccount, AbstractIbc, AbstractInterfaceError,
    Account, AccountFactory, AnsHost, ModuleFactory, VersionControl,
};
use abstract_std::{
    native_addrs::{self, TEST_ABSTRACT_CREATOR},
    ACCOUNT, ACCOUNT_FACTORY, ANS_HOST, IBC_CLIENT, IBC_HOST, MODULE_FACTORY, VERSION_CONTROL,
};

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
// Can't use symlinks in debug mode
// https://github.com/pyrossh/rust-embed/pull/234
#[folder = "./"]
#[include = "state.json"]
struct State;

impl State {
    pub fn load_state() -> serde_json::Value {
        let state_file =
            State::get("state.json").expect("Unable to read abstract-interface state.json");
        serde_json::from_slice(&state_file.data).unwrap()
    }
}

pub struct Abstract<Chain: CwEnv> {
    pub ans_host: AnsHost<Chain>,
    pub version_control: VersionControl<Chain>,
    pub account_factory: AccountFactory<Chain>,
    pub module_factory: ModuleFactory<Chain>,
    pub ibc: AbstractIbc<Chain>,
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
        let account = Account::new(ACCOUNT, chain.clone());

        let mut account = AbstractAccount { account: account };
        let ibc_infra = AbstractIbc::new(&chain);

        ans_host.upload()?;
        version_control.upload()?;
        account_factory.upload()?;
        module_factory.upload()?;
        account.upload()?;
        ibc_infra.upload()?;

        let deployment = Abstract {
            ans_host,
            account_factory,
            version_control,
            module_factory,
            account,
            ibc: ibc_infra,
        };

        Ok(deployment)
    }

    fn deploy_on(chain: Chain, data: Self::DeployData) -> Result<Self, AbstractInterfaceError> {
        // upload
        let mut deployment = Self::store_on(chain.clone())?;

        // ########### Instantiate ##############
        deployment.instantiate(data)?;

        // Set Factory
        deployment.version_control.execute(
            &abstract_std::version_control::ExecuteMsg::UpdateConfig {
                account_factory_address: Some(deployment.account_factory.address()?.into_string()),
                namespace_registration_fee: None,
                security_disabled: None,
            },
            &[],
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

        // Create the first abstract account in integration environments
        #[cfg(feature = "integration")]
        use abstract_std::objects::gov_type::GovernanceDetails;
        #[cfg(feature = "integration")]
        deployment
            .account_factory
            .create_default_account(GovernanceDetails::Monarchy {
                monarch: chain.sender_addr().to_string(),
            })?;
        Ok(deployment)
    }

    fn get_contracts_mut(&mut self) -> Vec<Box<&mut dyn ContractInstance<Chain>>> {
        vec![
            Box::new(&mut self.ans_host),
            Box::new(&mut self.version_control),
            Box::new(&mut self.account_factory),
            Box::new(&mut self.module_factory),
            Box::new(&mut self.account.account),
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
        if let Err(CwOrchError::AddrNotInStore(_)) = abstr.version_control.address() {
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
        let (ans_host, account_factory, version_control, module_factory) =
            get_native_contracts(chain.clone());
        let (ibc_client, ibc_host) = get_ibc_contracts(chain.clone());
        let account = Account::new(ACCOUNT, chain.clone());
        Self {
            account: AbstractAccount { account },
            ans_host,
            version_control,
            account_factory,
            module_factory,
            ibc: AbstractIbc {
                client: ibc_client,
                host: ibc_host,
            },
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

        self.version_control.instantiate(
            &abstract_std::version_control::InstantiateMsg {
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
                version_control_address: self.version_control.address()?.into_string(),
                ans_host_address: self.ans_host.address()?.into_string(),
            },
            Some(&admin),
            &[],
        )?;

        self.account_factory.instantiate(
            &abstract_std::account_factory::InstantiateMsg {
                admin: admin.to_string(),
                version_control_address: self.version_control.address()?.into_string(),
                ans_host_address: self.ans_host.address()?.into_string(),
                module_factory_address: self.module_factory.address()?.into_string(),
            },
            Some(&admin),
            &[],
        )?;

        // We also instantiate ibc contracts
        self.ibc.instantiate(self, &admin)?;
        self.ibc.register(&self.version_control)?;

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
            (
                self.ibc.host.as_instance(),
                ibc_host::contract::CONTRACT_VERSION.to_string(),
            ),
        ]
    }

    // Because of the mock tests limitations we expect that blob already uploaded
    pub fn deploy2(
        chain: Chain,
        deploy_data: <Self as Deploy<Chain>>::DeployData,
        blob_code_id: u64,
    ) -> Result<(), AbstractInterfaceError> {
        let admin = deploy_data.clone();
        // upload
        let deployment = Self::store_on(chain.clone())?;
        CwBlob::upload_and_migrate(
            chain.clone(),
            blob_code_id,
            &deployment.ans_host,
            &abstract_std::ans_host::MigrateMsg::Instantiate(
                abstract_std::ans_host::InstantiateMsg {
                    admin: admin.to_string(),
                },
            ),
            CanonicalAddr::from(native_addrs::ANS_ADDR),
            Binary::from(ANS_HOST.as_bytes()),
        );

        CwBlob::upload_and_migrate(
            chain.clone(),
            blob_code_id,
            &deployment.version_control,
            &abstract_std::version_control::MigrateMsg::Instantiate(
                abstract_std::version_control::InstantiateMsg {
                    admin: admin.to_string(),
                    #[cfg(feature = "integration")]
                    security_disabled: Some(true),
                    #[cfg(not(feature = "integration"))]
                    security_disabled: Some(false),
                    namespace_registration_fee: None,
                },
            ),
            CanonicalAddr::from(native_addrs::VERSION_CONTROL_ADDR),
            Binary::from(VERSION_CONTROL.as_bytes()),
        );

        CwBlob::upload_and_migrate(
            chain.clone(),
            blob_code_id,
            &deployment.module_factory,
            &abstract_std::module_factory::MigrateMsg::Instantiate(
                abstract_std::module_factory::InstantiateMsg {
                    admin: admin.to_string(),
                    version_control_address: deployment.version_control.address()?.into_string(),
                    ans_host_address: deployment.ans_host.address()?.into_string(),
                },
            ),
            CanonicalAddr::from(native_addrs::MODULE_FACTORY_ADDR),
            Binary::from(MODULE_FACTORY.as_bytes()),
        );

        CwBlob::upload_and_migrate(
            chain.clone(),
            blob_code_id,
            &deployment.account_factory,
            &abstract_std::account_factory::MigrateMsg::Instantiate(
                abstract_std::account_factory::InstantiateMsg {
                    admin: admin.to_string(),
                    version_control_address: deployment.version_control.address()?.into_string(),
                    ans_host_address: deployment.ans_host.address()?.into_string(),
                    module_factory_address: deployment.module_factory.address()?.into_string(),
                },
            ),
            CanonicalAddr::from(native_addrs::ACCOUNT_FACTORY_ADDR),
            Binary::from(ACCOUNT_FACTORY.as_bytes()),
        );

        // We also instantiate ibc contracts
        CwBlob::upload_and_migrate(
            chain.clone(),
            blob_code_id,
            &deployment.ibc.client,
            &abstract_std::ibc_client::MigrateMsg::Instantiate(
                abstract_std::ibc_client::InstantiateMsg {
                    ans_host_address: deployment.ans_host.addr_str()?,
                    version_control_address: deployment.version_control.addr_str()?,
                },
            ),
            CanonicalAddr::from(native_addrs::IBC_CLIENT_ADDR),
            Binary::from(IBC_CLIENT.as_bytes()),
        );
        CwBlob::upload_and_migrate(
            chain.clone(),
            blob_code_id,
            &deployment.ibc.host,
            &abstract_std::ibc_host::MigrateMsg::Instantiate(
                abstract_std::ibc_host::InstantiateMsg {
                    ans_host_address: deployment.ans_host.addr_str()?,
                    account_factory_address: deployment.account_factory.addr_str()?,
                    version_control_address: deployment.version_control.addr_str()?,
                },
            ),
            CanonicalAddr::from(native_addrs::IBC_HOST_ADDR),
            Binary::from(IBC_HOST.as_bytes()),
        );

        deployment.ibc.register(&deployment.version_control)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use std::borrow::Cow;

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
        let vc_juno = &state["juno"]["juno-1"]["code_ids"].get(VERSION_CONTROL);
        assert!(vc_juno.is_some());
    }
}
