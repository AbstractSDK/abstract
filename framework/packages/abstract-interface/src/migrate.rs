use abstract_core::{account_factory, ans_host, module_factory, version_control};
use cw_orch::prelude::*;

use crate::Abstract;

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

        // TODO: Add ibc client here

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

        self.version_control.approve_any_abstract_modules()?;

        Ok(account_factory.is_some()
            || module_factory.is_some()
            || version_control.is_some()
            || ans_host.is_some()
            || account)
    }
}
