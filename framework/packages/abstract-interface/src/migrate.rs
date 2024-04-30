use crate::{Abstract, AbstractIbc};
use abstract_std::version_control::QueryMsgFns;
use abstract_std::PROXY;
use abstract_std::{
    account_factory, ans_host, ibc_client, ibc_host, module_factory, objects::module::ModuleInfo,
    version_control, MANAGER,
};
use cosmwasm_std::from_json;
use cw2::{ContractVersion, CONTRACT};
use cw_orch::prelude::*;

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

        if ::account_factory::contract::CONTRACT_VERSION
            != contract_version(&self.account_factory)?.version
        {
            let migration_result = self
                .account_factory
                .upload_and_migrate_if_needed(&account_factory::MigrateMsg {})?;
            if migration_result.is_some() {
                has_migrated = true;
            }
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
        }

        if ::ans_host::contract::CONTRACT_VERSION != contract_version(&self.ans_host)?.version {
            let migration_result = self
                .ans_host
                .upload_and_migrate_if_needed(&ans_host::MigrateMsg {})?;
            if migration_result.is_some() {
                has_migrated = true;
            }
        }

        let mut modules_to_register = Vec::with_capacity(2);

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
            modules_to_register.push((
                self.account.manager.as_instance(),
                ::manager::contract::CONTRACT_VERSION.to_string(),
            ));
        }

        if ::proxy::contract::CONTRACT_VERSION != versions[1].module.info.version.to_string()
            && self.account.proxy.upload_if_needed()?.is_some()
        {
            modules_to_register.push((
                self.account.proxy.as_instance(),
                ::proxy::contract::CONTRACT_VERSION.to_string(),
            ));
        }

        if !modules_to_register.is_empty() {
            self.version_control
                .register_account_mods(modules_to_register)?;
            has_migrated = true
        }

        if self.ibc.migrate_if_version_changed()? {
            has_migrated = true
        }

        self.version_control.approve_any_abstract_modules()?;

        Ok(has_migrated)
    }
}

fn contract_version<Chain: CwEnv, A: ContractInstance<Chain>>(
    contract: &A,
) -> Result<ContractVersion, crate::AbstractInterfaceError> {
    let wasm_querier = contract.get_chain().wasm_querier();
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

    /// Migrate the ibc based on version changes. If the registered contracts have the right version, we don't migrate them
    pub fn migrate_if_version_changed(&self) -> Result<bool, crate::AbstractInterfaceError> {
        let mut has_migrated = false;

        if ::ibc_client::contract::CONTRACT_VERSION != contract_version(&self.client)?.version {
            let migration_result = self
                .client
                .upload_and_migrate_if_needed(&ibc_client::MigrateMsg {})?;
            if migration_result.is_some() {
                has_migrated = true;
            }
        }

        if ::ibc_host::contract::CONTRACT_VERSION != contract_version(&self.host)?.version {
            let migration_result = self
                .host
                .upload_and_migrate_if_needed(&ibc_host::MigrateMsg {})?;
            if migration_result.is_some() {
                has_migrated = true;
            }
        }

        Ok(has_migrated)
    }
}
