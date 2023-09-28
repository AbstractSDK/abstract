use crate::Abstract;
use abstract_core::objects::module::ModuleVersion;
use cw_orch::deploy::Deploy;
use cw_orch::prelude::CwOrchError::StdErr;
use cw_orch::prelude::*;

use semver::Version;
use serde::Serialize;

/// Trait for deploying Adapters
pub trait AdapterDeployer<Chain: CwEnv, CustomInitMsg: Serialize>: ContractInstance<Chain>
    + CwOrchInstantiate<Chain, InstantiateMsg = abstract_core::adapter::InstantiateMsg<CustomInitMsg>>
    + Uploadable
    + Sized
{
    /// Deploys the adapter. If the adapter is already deployed, it will return an error.
    /// Use `maybe_deploy` if you want to deploy the adapter only if it is not already deployed.
    fn deploy(
        &self,
        version: Version,
        custom_init_msg: CustomInitMsg,
    ) -> Result<(), crate::AbstractInterfaceError> {
        let deployed = self.maybe_deploy(version.clone(), custom_init_msg)?;
        if deployed {
            Ok(())
        } else {
            Err(StdErr(format!(
                "Adapter {} already exists with version {}",
                self.id(),
                version
            ))
            .into())
        }
    }

    /// Deploys the adapter if it is not already deployed
    /// Returns true if the adapter was deployed
    fn maybe_deploy(
        &self,
        version: Version,
        custom_init_msg: CustomInitMsg,
    ) -> Result<bool, crate::AbstractInterfaceError> {
        // retrieve the deployment
        let abstr = Abstract::load_from(self.get_chain().to_owned())?;

        // check for existing version
        let version_check = abstr
            .version_control
            .get_adapter_addr(&self.id(), ModuleVersion::from(version.to_string()));

        if version_check.is_ok() {
            return Ok(false);
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
            .register_adapters(vec![(self.as_instance(), version.to_string())])?;

        Ok(true)
    }
}

/// Trait for deploying APPs
pub trait AppDeployer<Chain: CwEnv>: Sized + Uploadable + ContractInstance<Chain> {
    /// Deploys the app. If the app is already deployed, it will return an error.
    /// Use `maybe_deploy` if you want to deploy the app only if it is not already deployed.
    fn deploy(&self, version: Version) -> Result<(), crate::AbstractInterfaceError> {
        let deployed = self.maybe_deploy(version.clone())?;
        if deployed {
            Ok(())
        } else {
            Err(StdErr(format!(
                "App {} already exists with version {}",
                self.id(),
                version
            ))
            .into())
        }
    }

    /// Deploys the application if it is not already deployed
    /// Returns true if the application was deployed
    fn maybe_deploy(&self, version: Version) -> Result<bool, crate::AbstractInterfaceError> {
        // retrieve the deployment
        let abstr = Abstract::<Chain>::load_from(self.get_chain().to_owned())?;

        // check for existing version
        let version_check = abstr
            .version_control
            .get_app_code(&self.id(), ModuleVersion::from(version.to_string()));

        if version_check.is_ok() {
            return Ok(false);
        };

        self.upload()?;

        abstr
            .version_control
            .register_apps(vec![(self.as_instance(), version.to_string())])?;

        Ok(true)
    }
}
