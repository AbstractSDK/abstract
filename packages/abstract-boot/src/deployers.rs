use crate::Abstract;
use abstract_core::objects::module::ModuleVersion;
use boot_core::{BootError::StdErr, CwEnv, Deploy, *};

use semver::Version;
use serde::Serialize;

/// Trait for deploying APIs
pub trait ApiDeployer<Chain: CwEnv, CustomInitMsg: Serialize>:
    ContractInstance<Chain>
    + BootInstantiate<Chain, InstantiateMsg = abstract_core::api::InstantiateMsg<CustomInitMsg>>
    + BootUpload<Chain>
{
    fn deploy(
        &mut self,
        version: Version,
        custom_init_msg: CustomInitMsg,
    ) -> Result<(), crate::AbstractBootError> {
        // retrieve the deployment
        let abstr = Abstract::load_from(self.get_chain().clone())?;

        // check for existing version
        let version_check = abstr
            .version_control
            .get_api_addr(&self.id(), ModuleVersion::from(version.to_string()));

        if version_check.is_ok() {
            return Err(StdErr(format!(
                "API {} already exists with version {}",
                self.id(),
                version
            ))
            .into());
        };

        self.upload()?;
        let init_msg = abstract_core::api::InstantiateMsg {
            module: custom_init_msg,
            base: abstract_core::api::BaseInstantiateMsg {
                ans_host_address: abstr.ans_host.address()?.into(),
                version_control_address: abstr.version_control.address()?.into(),
            },
        };
        self.instantiate(&init_msg, None, None)?;

        abstr
            .version_control
            .register_apis(vec![self.as_instance()], &version)?;
        Ok(())
    }
}

/// Trait for deploying APPs
pub trait AppDeployer<Chain: CwEnv>: ContractInstance<Chain> + BootUpload<Chain> {
    fn deploy(&mut self, version: Version) -> Result<(), crate::AbstractBootError> {
        // retrieve the deployment
        let abstr = Abstract::load_from(self.get_chain().clone())?;

        // check for existing version
        let version_check = abstr
            .version_control
            .get_app_code(&self.id(), ModuleVersion::from(version.to_string()));

        if version_check.is_ok() {
            return Err(StdErr(format!(
                "API {} already exists with version {}",
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
