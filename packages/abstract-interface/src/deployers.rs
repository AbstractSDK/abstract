use crate::Abstract;
use abstract_core::objects::module::ModuleVersion;
use cw_orch::deploy::Deploy;
use cw_orch::environment::ChainUpload;
use cw_orch::prelude::CwOrchError::StdErr;
use cw_orch::prelude::*;

use semver::Version;
use serde::Serialize;

/// Trait for deploying Adapters
pub trait AdapterDeployer<Chain: CwEnv + ChainUpload, CustomInitMsg: Serialize>:
    ContractInstance<Chain>
    + CwOrcInstantiate<Chain, InstantiateMsg = abstract_core::adapter::InstantiateMsg<CustomInitMsg>>
    + CwOrcUpload<Chain>
{
    fn deploy(
        &self,
        version: Version,
        custom_init_msg: CustomInitMsg,
    ) -> Result<(), crate::AbstractInterfaceError> {
        // retrieve the deployment
        let abstr = Abstract::load_from(self.get_chain().to_owned())?;

        // check for existing version
        let version_check = abstr
            .version_control
            .get_adapter_addr(&self.id(), ModuleVersion::from(version.to_string()));

        if version_check.is_ok() {
            return Err(StdErr(format!(
                "Adapter {} already exists with version {}",
                self.id(),
                version
            ))
            .into());
        };

        self.upload()?;
        let init_msg = abstract_core::adapter::InstantiateMsg {
            module: custom_init_msg,
            base: abstract_core::adapter::BaseInstantiateMsg {
                ans_host_address: abstr.ans_host.address()?.into(),
                version_control_address: abstr.version_control.address()?.into(),
            },
        };
        self.instantiate(&init_msg, None, None)?;

        abstr
            .version_control
            .register_adapters(vec![self.as_instance()], &version)?;
        Ok(())
    }
}

/// Trait for deploying APPs
pub trait AppDeployer<Chain: CwEnv + ChainUpload>:
    ContractInstance<Chain> + CwOrcUpload<Chain>
{
    fn deploy(&mut self, version: Version) -> Result<(), crate::AbstractInterfaceError> {
        // retrieve the deployment
        let abstr = Abstract::<Chain>::load_from(self.get_chain().to_owned())?;

        // check for existing version
        let version_check = abstr
            .version_control
            .get_app_code(&self.id(), ModuleVersion::from(version.to_string()));

        if version_check.is_ok() {
            return Err(StdErr(format!(
                "App {} already exists with version {}",
                self.id(),
                version
            ))
            .into());
        };

        self.upload()?;

        abstr
            .version_control
            .register_apps(vec![self.as_instance()], &version)?;
        Ok(())
    }
}
