use std::path::PathBuf;

use abstract_std::{
    account_factory::{ExecuteMsgFns as _, QueryMsgFns},
    profile, ACCOUNT_FACTORY, ANS_HOST, MANAGER, MODULE_FACTORY, PROFILE, PROFILE_MARKETPLACE,
    PROXY, VERSION_CONTROL,
};
use bs721::{CollectionInfo, RoyaltyInfoResponse};
use bs_profile::Metadata;
use cw_orch::prelude::*;

use crate::{
    get_ibc_contracts, get_native_contracts, AbstractAccount, AbstractIbc, AbstractInterfaceError,
    AccountFactory, AnsHost, Manager, ModuleFactory, Profile, ProfileMarketplace, Proxy,
    VersionControl,
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
    // bitsong profile contracts
    pub bs721_profile: Profile<Chain, Metadata>,
    pub profile_marketplace: ProfileMarketplace<Chain>,
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
        let bs721_profile: Profile<Chain, Metadata> = Profile::new(PROFILE, chain.clone());
        let profile_marketplace = ProfileMarketplace::new(PROFILE_MARKETPLACE, chain.clone());

        let mut account = AbstractAccount { manager, proxy };
        let ibc_infra = AbstractIbc::new(&chain);

        ans_host.upload()?;
        version_control.upload()?;
        account_factory.upload()?;
        module_factory.upload()?;
        account.upload()?;
        ibc_infra.upload()?;
        bs721_profile.upload()?;
        profile_marketplace.upload()?;

        let res1 = ans_host.upload()?;
        let res2 = version_control.upload()?;
        let res3 = account_factory.upload()?;
        let res4 = module_factory.upload()?;
        let res5 = account.upload()?;
        let res6 = ibc_infra.upload()?;
        let res7 = profile_marketplace.upload()?;
        let res8 = bs721_profile.upload()?;
        println!("ANS Host upload result: {:?}", res1);
        println!("Version Control upload result: {:?}", res2);
        println!("ANS Host upload result: {:?}", res3);
        println!("Account Factory upload result: {:?}", res3);
        println!("Module Factory upload result: {:?}", res4);
        println!("Account  upload result: {:?}", res5);
        println!("Profile  upload result: {:?}", res6);
        println!("Marketplace  upload result: {:?}", res7);
        println!("Bs721Profile  upload result: {:?}", res8);

        let deployment = Abstract {
            ans_host,
            account_factory,
            version_control,
            module_factory,
            account,
            ibc: ibc_infra,
            profile_marketplace,
            bs721_profile,
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
            &abstract_std::version_control::ExecuteMsg::UpdateConfig {
                account_factory_address: Some(deployment.account_factory.address()?.into_string()),
                namespace_registration_fee: None,
                security_disabled: None,
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
                None,
                None,
                None,
                None,
            )
            .unwrap();

        deployment
            .account_factory
            .setup_profile_infra(
                deployment.profile_marketplace.code_id()?,
                deployment.bs721_profile.code_id()?,
            )
            .unwrap();

        let profile_config = deployment.account_factory.profile_config()?;

        deployment
            .bs721_profile
            .set_address(&Addr::unchecked(profile_config.collection_addr.unwrap()));
        deployment
            .profile_marketplace
            .set_address(&Addr::unchecked(profile_config.marketplace_addr.unwrap()));

        // Create the first abstract account in integration environments
        #[cfg(feature = "integration")]
        use abstract_std::objects::gov_type::GovernanceDetails;
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
            Box::new(&mut self.ibc.client),
            Box::new(&mut self.ibc.host),
            Box::new(&mut self.bs721_profile),
            Box::new(&mut self.profile_marketplace),
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
        let state = State::load_state();
        abstr.set_contracts_state(Some(state));

        // Check if abstract deployed, for successful load
        if let Err(CwOrchError::AddrNotInStore(_)) = abstr.version_control.address() {
            return Err(AbstractInterfaceError::NotDeployed {});
        }
        Ok(abstr)
    }
}

impl<Chain: CwEnv> Abstract<Chain> {
    pub fn new(chain: Chain) -> Self {
        let (
            ans_host,
            account_factory,
            version_control,
            module_factory,
            bs721_profile,
            profile_marketplace,
        ) = get_native_contracts(chain.clone());
        let (ibc_client, ibc_host) = get_ibc_contracts(chain.clone());
        let manager = Manager::new(MANAGER, chain.clone());
        let proxy = Proxy::new(PROXY, chain);
        Self {
            account: AbstractAccount { manager, proxy },
            ans_host,
            version_control,
            account_factory,
            module_factory,
            ibc: AbstractIbc {
                client: ibc_client,
                host: ibc_host,
            },
            profile_marketplace,
            bs721_profile,
        }
    }

    pub fn instantiate(
        &mut self,
        _chain: &Chain,
        admin: String,
    ) -> Result<(), AbstractInterfaceError> {
        let admin = Addr::unchecked(admin);

        self.ans_host.instantiate(
            &abstract_std::ans_host::InstantiateMsg {
                admin: admin.to_string(),
            },
            Some(&admin),
            None,
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
            None,
        )?;

        self.module_factory.instantiate(
            &abstract_std::module_factory::InstantiateMsg {
                admin: admin.to_string(),
                version_control_address: self.version_control.address()?.into_string(),
                ans_host_address: self.ans_host.address()?.into_string(),
            },
            Some(&admin),
            None,
        )?;

        self.account_factory.instantiate(
            &abstract_std::account_factory::InstantiateMsg {
                admin: admin.to_string(),
                version_control_address: self.version_control.address()?.into_string(),
                ans_host_address: self.ans_host.address()?.into_string(),
                module_factory_address: self.module_factory.address()?.into_string(),
                max_record_count: None,
                max_profile_length: None,
                min_profile_length: None,
                verifier: None,
                profile_bps: None,
            },
            Some(&admin),
            None,
        )?;
        println!(
            "Account Factory Contract: {:?}",
            self.account_factory.address()?.to_string()
        );

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
            // (
            //     self.bs721_profile.as_instance(),
            //     bs721_base::CONTRACT_VERSION.to_string(),
            // ),
            // (
            //     self.profile_marketplace.as_instance(),
            //     profile_marketplace::contract::CONTRACT_VERSION.to_string(),
            // )
        ]
    }
}

#[cfg(test)]
mod test {
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
