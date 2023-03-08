use crate::Abstract;
use abstract_os::objects::module::ModuleVersion;
use boot_core::{BootEnvironment, BootError::StdErr, Deploy, *};

use semver::Version;
use serde::Serialize;

/// Trait for deploying APIs
pub trait ApiDeployer<Chain: BootEnvironment, CustomInitMsg: Serialize>:
    ContractInstance<Chain>
    + BootInstantiate<Chain, InstantiateMsg = abstract_os::api::InstantiateMsg<CustomInitMsg>>
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
        let init_msg = abstract_os::api::InstantiateMsg {
            app: custom_init_msg,
            base: abstract_os::api::BaseInstantiateMsg {
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
pub trait AppDeployer<Chain: BootEnvironment>: ContractInstance<Chain> + BootUpload<Chain> {
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

// /// An Abstract module deployer that can deploy modules to a chain.
// pub struct ModuleDeployer<Chain: BootEnvironment> {
//     pub chain: Chain,
//     pub version: Version,
//     pub ans_host: AnsHost<Chain>,
//     pub version_control: VersionControl<Chain>,
// }

// impl<Chain: BootEnvironment> ModuleDeployer<Chain> {
//     /// Create a new instance of the module deployer, loaded from the STATE_FILE.
//     pub fn new(chain: Chain, version: Version) -> Self {
//         let ans_host = AnsHost::new(ANS_HOST, chain.clone());
//         let version_control = VersionControl::new(VERSION_CONTROL, chain.clone());
//         Self {
//             chain,
//             ans_host,
//             version_control,
//             version,
//         }
//     }
//     /// Loads a deployment instance from a live chain given the **version_control_address**.
//     pub fn load_from_version_control(
//         chain: Chain,
//         abstract_version: &Version,
//         version_control_address: &Addr,
//     ) -> Result<Self, crate::AbstractBootError> {
//         let version_control = VersionControl::load(chain.clone(), version_control_address);

//         // TODO: get the version dynamically
//         // let info = &self.chain.runtime.block_on(DaemonQuerier::contract_info(
//         //     chain.sender.channel(),
//         //     self.address()?,
//         // ))?;

//         let result = version_control.get_api_addr(ANS_HOST, ModuleVersion::Latest);

//         let ans_host = AnsHost::load(chain.clone(), &result?);

//         Ok(Self {
//             chain,
//             ans_host,
//             version_control,
//             version: abstract_version.clone(),
//         })
//     }

//     /// Uploads, instantiates, and registers a new API module.
//     pub fn deploy_api<TInitMsg>(
//         &self,
//         api: &mut Contract<Chain>,
//         version: Version,
//         api_init_msg: TInitMsg,
//     ) -> Result<(), crate::AbstractBootError>
//     where
//         TInitMsg: Serialize + Debug,
//     {
//         // check for existing version
//         let version_check = self
//             .version_control
//             .get_api_addr(&api.id, ModuleVersion::from(version.to_string()));

//         if version_check.is_ok() {
//             return Err(StdErr(format!(
//                 "API {} already exists with version {}",
//                 api.id, version
//             ))
//             .into());
//         };

//         api.upload()?;
//         let init_msg = abstract_os::api::InstantiateMsg {
//             app: api_init_msg,
//             base: abstract_os::api::BaseInstantiateMsg {
//                 ans_host_address: self.ans_host.address()?.into(),
//                 version_control_address: self.version_control.address()?.into(),
//             },
//         };
//         api.instantiate(&init_msg, None, None)?;

//         self.version_control.register_apis(vec![api], &version)?;
//         Ok(())
//     }
// }
