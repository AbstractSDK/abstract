use crate::Abstract;
use abstract_core::{account_factory, ans_host, module_factory, version_control};
use cw_orch::daemon::ConditionalMigrate;
use cw_orch::prelude::*;

impl Abstract<Daemon> {
    /// Migrate the deployment based on the uploaded and local wasm files. If the remote wasm file is older, upload the contract and migrate to the new version.
    pub fn migrate(&self) -> Result<(), crate::AbstractInterfaceError> {
        // start with factories
        self.account_factory
            .upload_and_migrate_if_needed(&account_factory::MigrateMsg {})?;
        self.module_factory
            .upload_and_migrate_if_needed(&module_factory::MigrateMsg {})?;

        // TODO: Add ibc client here

        // then VC and ANS
        self.version_control
            .upload_and_migrate_if_needed(&version_control::MigrateMsg {})?;
        self.ans_host
            .upload_and_migrate_if_needed(&ans_host::MigrateMsg {})?;

        // Then upload and register account if needed
        self.account
            .upload_and_register_if_needed(&self.version_control)?;

        Ok(())
    }
}
