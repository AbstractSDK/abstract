use crate::{Abstract, AbstractIbc};
use abstract_std::ibc_client::QueryMsgFns as _;
use abstract_std::version_control::QueryMsgFns;
use abstract_std::{
    account_factory, ans_host, ibc_client, ibc_host, module_factory, objects::module::ModuleInfo,
    version_control, MANAGER,
};
use abstract_std::{
    ACCOUNT_FACTORY, ANS_HOST, IBC_CLIENT, IBC_HOST, MODULE_FACTORY, PROXY, VERSION_CONTROL,
};
use cosmwasm_std::from_json;
use cw2::{ContractVersion, CONTRACT};
use cw_orch::{environment::Environment, prelude::*};

impl<T: CwEnv> Abstract<T> {
    /// Migrate the deployment based on the uploaded and local wasm files. If the remote wasm file is older, upload the contract and migrate to the new version.
    pub fn migrate_if_needed(&self) -> Result<bool, crate::AbstractInterfaceError> {
        // start with factories
        let account_factory = self
            .account_factory
            .upload_and_migrate_if_needed(&account_factory::MigrateMsg {})?;
        let module_factory = self
            .module_factory
            .upload_and_migrate_if_needed(&module_factory::MigrateMsg {})?;

        // then VC and ANS
        let version_control = self
            .version_control
            .upload_and_migrate_if_needed(&version_control::MigrateMsg {})?;
        let ans_host = self
            .ans_host
            .upload_and_migrate_if_needed(&ans_host::MigrateMsg {})?;

        // Then upload and register account if needed
        let account = self
            .account
            .upload_and_register_if_needed(&self.version_control)?;

        // Then ibc
        let ibc = self.ibc.migrate_if_needed()?;

        self.version_control.approve_any_abstract_modules()?;

        Ok(account_factory.is_some()
            || module_factory.is_some()
            || version_control.is_some()
            || ans_host.is_some()
            || account
            || ibc)
    }

    /// Migrate the deployment based on version changes. If the registered contracts have the right version, we don't migrate them
    pub fn migrate_if_version_changed(&self) -> Result<bool, crate::AbstractInterfaceError> {
        let mut has_migrated = false;
        let mut natives_to_register = vec![];

        if ::account_factory::contract::CONTRACT_VERSION
            != contract_version(&self.account_factory)?.version
        {
            let migration_result = self
                .account_factory
                .upload_and_migrate_if_needed(&account_factory::MigrateMsg {})?;
            if migration_result.is_some() {
                has_migrated = true;
            }
            natives_to_register.push((
                self.account_factory.as_instance(),
                ::account_factory::contract::CONTRACT_VERSION.to_string(),
            ));
        }
        if ::module_factory::contract::CONTRACT_VERSION
            != contract_version(&self.module_factory)?.version
        {
            let migration_result = self
                .module_factory
                .upload_and_migrate_if_needed(&module_factory::MigrateMsg {})?;
            if migration_result.is_some() {
                has_migrated = true;
            }
            natives_to_register.push((
                self.module_factory.as_instance(),
                ::module_factory::contract::CONTRACT_VERSION.to_string(),
            ));
        }

        if ::version_control::contract::CONTRACT_VERSION
            != contract_version(&self.version_control)?.version
        {
            let migration_result = self
                .version_control
                .upload_and_migrate_if_needed(&version_control::MigrateMsg {})?;
            if migration_result.is_some() {
                has_migrated = true;
            }
            natives_to_register.push((
                self.version_control.as_instance(),
                ::version_control::contract::CONTRACT_VERSION.to_string(),
            ));
        }

        if ::ans_host::contract::CONTRACT_VERSION != contract_version(&self.ans_host)?.version {
            let migration_result = self
                .ans_host
                .upload_and_migrate_if_needed(&ans_host::MigrateMsg {})?;
            if migration_result.is_some() {
                has_migrated = true;
            }
            natives_to_register.push((
                self.ans_host.as_instance(),
                ::ans_host::contract::CONTRACT_VERSION.to_string(),
            ));
        }

        let mut accounts_to_register = Vec::with_capacity(2);

        // We need to check the version in version control for the account contracts
        let versions = self
            .version_control
            .modules(vec![
                ModuleInfo::from_id_latest(MANAGER)?,
                ModuleInfo::from_id_latest(PROXY)?,
            ])?
            .modules;

        if ::manager::contract::CONTRACT_VERSION != versions[0].module.info.version.to_string()
            && self.account.manager.upload_if_needed()?.is_some()
        {
            accounts_to_register.push((
                self.account.manager.as_instance(),
                ::manager::contract::CONTRACT_VERSION.to_string(),
            ));
        }

        if ::proxy::contract::CONTRACT_VERSION != versions[1].module.info.version.to_string()
            && self.account.proxy.upload_if_needed()?.is_some()
        {
            accounts_to_register.push((
                self.account.proxy.as_instance(),
                ::proxy::contract::CONTRACT_VERSION.to_string(),
            ));
        }

        if !accounts_to_register.is_empty() {
            self.version_control
                .register_account_mods(accounts_to_register)?;
            has_migrated = true
        }

        if self.ibc.deploy_or_migrate_if_version_changed(self)? {
            has_migrated = true;

            natives_to_register.push((
                self.ibc.client.as_instance(),
                ::ibc_client::contract::CONTRACT_VERSION.to_string(),
            ));
            natives_to_register.push((
                self.ibc.host.as_instance(),
                ::ibc_host::contract::CONTRACT_VERSION.to_string(),
            ));
        }

        self.version_control.register_natives(natives_to_register)?;
        self.version_control.approve_any_abstract_modules()?;

        Ok(has_migrated)
    }

    /// Registers the deployment in version control  
    pub fn register_in_version_control(&self) -> Result<(), crate::AbstractInterfaceError> {
        let mut natives_to_register = vec![];

        let modules = self
            .version_control
            .modules(vec![
                ModuleInfo::from_id_latest(ACCOUNT_FACTORY)?,
                ModuleInfo::from_id_latest(MODULE_FACTORY)?,
                ModuleInfo::from_id_latest(VERSION_CONTROL)?,
                ModuleInfo::from_id_latest(ANS_HOST)?,
                ModuleInfo::from_id_latest(IBC_CLIENT)?,
                ModuleInfo::from_id_latest(IBC_HOST)?,
            ])?
            .modules;

        let account_factory_module = modules[0].module.clone();
        let module_factory_module = modules[1].module.clone();
        let version_control_module = modules[2].module.clone();
        let ans_host_module = modules[3].module.clone();
        let ibc_client_module = modules[4].module.clone();
        let ibc_host_module = modules[5].module.clone();

        // In case cw2 debugging required
        // let account_factory_cw2_v = contract_version(&self.account_factory)?.version;
        // let module_factory_cw2_v = contract_version(&self.module_factory)?.version;
        // let version_control_cw2_v = contract_version(&self.version_control)?.version;
        // let ans_host_cw2_v = contract_version(&self.ans_host)?.version;
        // let ibc_client_cw2_v = contract_version(&self.ibc.client)?.version;
        // let ibc_host_cw2_v = contract_version(&self.ibc.host)?.version;
        // let versions = vec![
        //     account_factory_cw2_v,
        //     module_factory_cw2_v,
        //     version_control_cw2_v,
        //     ans_host_cw2_v,
        //     ibc_client_cw2_v,
        //     ibc_host_cw2_v,
        // ];
        // panic!("{versions:?}");

        if ::account_factory::contract::CONTRACT_VERSION
            != account_factory_module.info.version.to_string()
        {
            natives_to_register.push((
                self.account_factory.as_instance(),
                ::account_factory::contract::CONTRACT_VERSION.to_string(),
            ));
        }

        if ::module_factory::contract::CONTRACT_VERSION
            != module_factory_module.info.version.to_string()
        {
            natives_to_register.push((
                self.module_factory.as_instance(),
                ::module_factory::contract::CONTRACT_VERSION.to_string(),
            ));
        }

        if ::version_control::contract::CONTRACT_VERSION
            != version_control_module.info.version.to_string()
        {
            natives_to_register.push((
                self.version_control.as_instance(),
                ::version_control::contract::CONTRACT_VERSION.to_string(),
            ));
        }

        if ::ans_host::contract::CONTRACT_VERSION != ans_host_module.info.version.to_string() {
            natives_to_register.push((
                self.ans_host.as_instance(),
                ::ans_host::contract::CONTRACT_VERSION.to_string(),
            ));
        }

        if ::ibc_client::contract::CONTRACT_VERSION != ibc_client_module.info.version.to_string() {
            natives_to_register.push((
                self.ibc.client.as_instance(),
                ::ibc_client::contract::CONTRACT_VERSION.to_string(),
            ));
        }

        if ::ibc_host::contract::CONTRACT_VERSION != ibc_host_module.info.version.to_string() {
            natives_to_register.push((
                self.ibc.host.as_instance(),
                ::ibc_host::contract::CONTRACT_VERSION.to_string(),
            ));
        }

        self.version_control.register_natives(natives_to_register)?;
        self.version_control.approve_any_abstract_modules()?;

        Ok(())
    }
}

fn contract_version<Chain: CwEnv, A: ContractInstance<Chain>>(
    contract: &A,
) -> Result<ContractVersion, crate::AbstractInterfaceError> {
    let wasm_querier = contract.environment().wasm_querier();
    Ok(from_json(
        wasm_querier
            .raw_query(
                contract.address()?.to_string(),
                CONTRACT.as_slice().to_vec(),
            )
            .unwrap(),
    )?)
}

impl<Chain: CwEnv> AbstractIbc<Chain> {
    /// Migrate the deployment based on the uploaded and local wasm files. If the remote wasm file is older, upload the contract and migrate to the new version.
    pub fn migrate_if_needed(&self) -> Result<bool, crate::AbstractInterfaceError> {
        let client = self
            .client
            .upload_and_migrate_if_needed(&ibc_client::MigrateMsg {})?;
        let host = self
            .host
            .upload_and_migrate_if_needed(&ibc_host::MigrateMsg {})?;
        Ok(client.is_some() || host.is_some())
    }

    /// Deploys or Migrates IBC Client and IBC Host
    /// - If no version changes - don't do anything
    /// - If version change is breaking - new version is deployed
    /// - If version change is non-breaking - ibc contracts migrated instead
    pub fn deploy_or_migrate_if_version_changed(
        &self,
        abstr: &Abstract<Chain>,
    ) -> Result<bool, crate::AbstractInterfaceError> {
        let ibc_client_cw2_version = contract_version(&self.client)?.version;
        // Check if any version changes
        // *Note: IBC client and IBC host supposed to be versioned equally
        if ::ibc_client::contract::CONTRACT_VERSION == ibc_client_cw2_version {
            // No need to do anything
            return Ok(false);
        }

        // Version change - upload both contracts
        self.client
            .upload_if_needed()?
            .expect("IBC client wasm might be outdated");
        self.host
            .upload_if_needed()?
            .expect("IBC host wasm might be outdated");

        // Check if version is breaking
        let version_req = cw_semver::VersionReq::parse(&ibc_client_cw2_version).unwrap();
        let new_version =
            cw_semver::Version::parse(::ibc_client::contract::CONTRACT_VERSION).unwrap();
        if version_req.matches(&new_version) {
            // If version is not breaking, simply migrate
            self.client
                .migrate_if_needed(&ibc_client::MigrateMsg {})?
                .expect("IBC client supposed to be migrated, but skipped instead");
            self.host
                .migrate_if_needed(&ibc_host::MigrateMsg {})?
                .expect("IBC host supposed to be migrated, but skipped instead");
        } else {
            // Version change is breaking, need to deploy new version
            let infrastructures = self.client.list_ibc_infrastructures()?;
            self.instantiate(
                abstr,
                &self.client.environment().sender_addr(),
                // Copy previous polytone connections
                infrastructures,
            )?;
        }

        Ok(true)
    }
}
