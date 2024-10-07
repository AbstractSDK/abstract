use crate::{Abstract, AbstractIbc};
use abstract_std::registry::QueryMsgFns;
use abstract_std::{
    ans_host, ibc_client, ibc_host, module_factory, objects::module::ModuleInfo, registry, ACCOUNT,
};
use abstract_std::{ANS_HOST, IBC_CLIENT, IBC_HOST, MODULE_FACTORY, REGISTRY};
use cosmwasm_std::from_json;
use cw2::{ContractVersion, CONTRACT};
use cw_orch::{environment::Environment, prelude::*};

impl<T: CwEnv> Abstract<T> {
    /// Migrate the deployment based on the uploaded and local wasm files. If the remote wasm file is older, upload the contract and migrate to the new version.
    pub fn migrate_if_needed(&self) -> Result<bool, crate::AbstractInterfaceError> {
        // start with factory
        let module_factory = self
            .module_factory
            .upload_and_migrate_if_needed(&module_factory::MigrateMsg::Migrate {})?;

        // then VC and ANS
        let registry = self
            .registry
            .upload_and_migrate_if_needed(&registry::MigrateMsg::Migrate {})?;
        let ans_host = self
            .ans_host
            .upload_and_migrate_if_needed(&ans_host::MigrateMsg::Migrate {})?;

        // Then upload and register account if needed
        let account = self.account.upload_and_register_if_needed(&self.registry)?;

        // Then ibc
        let ibc = self.ibc.migrate_if_needed()?;

        self.registry.approve_any_abstract_modules()?;

        Ok(module_factory.is_some() || registry.is_some() || ans_host.is_some() || account || ibc)
    }

    /// Migrate the deployment based on version changes. If the registered contracts have the right version, we don't migrate them
    pub fn migrate_if_version_changed(&self) -> Result<bool, crate::AbstractInterfaceError> {
        let mut has_migrated = false;
        let mut natives_to_register = vec![];

        if ::module_factory::contract::CONTRACT_VERSION
            != contract_version(&self.module_factory)?.version
        {
            let migration_result = self
                .module_factory
                .upload_and_migrate_if_needed(&module_factory::MigrateMsg::Migrate {})?;
            if migration_result.is_some() {
                has_migrated = true;
            }
            natives_to_register.push((
                self.module_factory.as_instance(),
                ::module_factory::contract::CONTRACT_VERSION.to_string(),
            ));
        }

        if ::registry::contract::CONTRACT_VERSION != contract_version(&self.registry)?.version {
            let migration_result = self
                .registry
                .upload_and_migrate_if_needed(&registry::MigrateMsg::Migrate {})?;
            if migration_result.is_some() {
                has_migrated = true;
            }
            natives_to_register.push((
                self.registry.as_instance(),
                ::registry::contract::CONTRACT_VERSION.to_string(),
            ));
        }

        if ::ans_host::contract::CONTRACT_VERSION != contract_version(&self.ans_host)?.version {
            let migration_result = self
                .ans_host
                .upload_and_migrate_if_needed(&ans_host::MigrateMsg::Migrate {})?;
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
            .registry
            .modules(vec![
                ModuleInfo::from_id_latest(ACCOUNT)?,
                ModuleInfo::from_id_latest(ACCOUNT)?,
            ])?
            .modules;

        if ::account::contract::CONTRACT_VERSION != versions[0].module.info.version.to_string()
            && self.account.upload_if_needed()?.is_some()
        {
            accounts_to_register.push((
                self.account.as_instance(),
                ::account::contract::CONTRACT_VERSION.to_string(),
            ));
        }

        if !accounts_to_register.is_empty() {
            self.registry.register_account_mods(accounts_to_register)?;
            has_migrated = true
        }

        if self.ibc.deploy_or_migrate_if_version_changed()? {
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

        self.registry.register_natives(natives_to_register)?;
        self.registry.approve_any_abstract_modules()?;

        Ok(has_migrated)
    }

    /// Registers the deployment in version control  
    pub fn register_in_registry(&self) -> Result<(), crate::AbstractInterfaceError> {
        let mut natives_to_register = vec![];

        let modules = self
            .registry
            .modules(vec![
                ModuleInfo::from_id_latest(MODULE_FACTORY)?,
                ModuleInfo::from_id_latest(REGISTRY)?,
                ModuleInfo::from_id_latest(ANS_HOST)?,
                ModuleInfo::from_id_latest(IBC_CLIENT)?,
                ModuleInfo::from_id_latest(IBC_HOST)?,
            ])?
            .modules;

        let module_factory_module = modules[0].module.clone();
        let registry_module = modules[1].module.clone();
        let ans_host_module = modules[2].module.clone();
        let ibc_client_module = modules[3].module.clone();
        let ibc_host_module = modules[4].module.clone();

        // In case cw2 debugging required
        // let module_factory_cw2_v = contract_version(&self.module_factory)?.version;
        // let registry_cw2_v = contract_version(&self.registry)?.version;
        // let ans_host_cw2_v = contract_version(&self.ans_host)?.version;
        // let ibc_client_cw2_v = contract_version(&self.ibc.client)?.version;
        // let ibc_host_cw2_v = contract_version(&self.ibc.host)?.version;
        // let versions = vec![
        //     module_factory_cw2_v,
        //     registry_cw2_v,
        //     ans_host_cw2_v,
        //     ibc_client_cw2_v,
        //     ibc_host_cw2_v,
        // ];
        // panic!("{versions:?}");

        if ::module_factory::contract::CONTRACT_VERSION
            != module_factory_module.info.version.to_string()
        {
            natives_to_register.push((
                self.module_factory.as_instance(),
                ::module_factory::contract::CONTRACT_VERSION.to_string(),
            ));
        }

        if ::registry::contract::CONTRACT_VERSION != registry_module.info.version.to_string() {
            natives_to_register.push((
                self.registry.as_instance(),
                ::registry::contract::CONTRACT_VERSION.to_string(),
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

        self.registry.register_natives(natives_to_register)?;
        self.registry.approve_any_abstract_modules()?;

        Ok(())
    }
}

fn contract_version<Chain: CwEnv, A: ContractInstance<Chain>>(
    contract: &A,
) -> Result<ContractVersion, crate::AbstractInterfaceError> {
    let wasm_querier = contract.environment().wasm_querier();
    Ok(from_json(
        wasm_querier
            .raw_query(&contract.address()?, CONTRACT.as_slice().to_vec())
            .unwrap(),
    )?)
}

impl<Chain: CwEnv> AbstractIbc<Chain> {
    /// Migrate the deployment based on the uploaded and local wasm files. If the remote wasm file is older, upload the contract and migrate to the new version.
    pub fn migrate_if_needed(&self) -> Result<bool, crate::AbstractInterfaceError> {
        let client = self
            .client
            .upload_and_migrate_if_needed(&ibc_client::MigrateMsg::Migrate {})?;
        let host = self
            .host
            .upload_and_migrate_if_needed(&ibc_host::MigrateMsg::Migrate {})?;
        Ok(client.is_some() || host.is_some())
    }

    /// Deploys or Migrates IBC Client and IBC Host
    /// - If no version changes - don't do anything
    /// - If version change is breaking - new version is deployed
    /// - If version change is non-breaking - ibc contracts migrated instead
    pub fn deploy_or_migrate_if_version_changed(
        &self,
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
        let version_req = semver::VersionReq::parse(&ibc_client_cw2_version).unwrap();
        let new_version = semver::Version::parse(::ibc_client::contract::CONTRACT_VERSION).unwrap();
        if version_req.matches(&new_version) {
            // If version is not breaking, simply migrate
            self.client
                .migrate_if_needed(&ibc_client::MigrateMsg::Migrate {})?
                .expect("IBC client supposed to be migrated, but skipped instead");
            self.host
                .migrate_if_needed(&ibc_host::MigrateMsg::Migrate {})?
                .expect("IBC host supposed to be migrated, but skipped instead");
        } else {
            // Version change is breaking, need to deploy new version
            self.instantiate(&self.client.environment().sender_addr())?;
        }

        Ok(true)
    }
}
