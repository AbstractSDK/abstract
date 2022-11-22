use std::fmt::Debug;

use boot_core::interface::{BootExecute, BootQuery, ContractInstance};
use boot_core::prelude::boot_contract;
use boot_core::{state::StateInterface, BootEnvironment, BootError, Contract, Daemon, TxResponse, IndexResponse};
use cosmwasm_std::Addr;
use semver::Version;
use serde::Serialize;

use abstract_sdk::os::{
    extension,
    objects::{
        module::{ModuleInfo, ModuleVersion},
        module_reference::ModuleReference,
    },
    registry,
    version_control::*,
    VERSION_CONTROL,
};

#[boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct VersionControl<Chain>;

impl<Chain: BootEnvironment> VersionControl<Chain> where TxResponse<Chain>: IndexResponse {
    pub fn new(name: &str, chain: &Chain) -> Self {
        Self(
            Contract::new(name, chain).with_wasm_path("version_control"),
            // .with_mock(Box::new(
            //     ContractWrapper::new_with_empty(
            //         ::contract::execute,
            //         ::contract::instantiate,
            //         ::contract::query,
            //     ),
            // ))
        )
    }

    pub fn load(chain: &Chain, address: &Addr) -> Self {
        Self(Contract::new(VERSION_CONTROL, chain).with_address(Some(address)))
    }

    pub fn upload_and_register_module(
        &self,
        module: &mut Contract<Chain>,
        new_version: &Version,
    ) -> Result<(), BootError> {
        module.upload()?;
        self.execute(
            &ExecuteMsg::AddModules {
                modules: vec![(
                    ModuleInfo::from_id(
                        &module.id,
                        ModuleVersion::Version(new_version.to_string()),
                    )?,
                    ModuleReference::App(module.code_id()?),
                )],
            },
            None,
        )?;
        log::info!("Module {} uploaded and registered", module.id);
        Ok(())
    }

    pub fn upload_and_register_extension<AppMsg: Serialize + Debug>(
        &self,
        extension: &mut Contract<Chain>,
        // extension: &mut Contract<Chain, E, extension::InstantiateMsg<AppMsg>, Q, M>,
        extension_init_msg: &extension::InstantiateMsg<AppMsg>,
        new_version: &Version,
    ) -> Result<(), BootError> {
        extension.upload()?;
        extension.instantiate(extension_init_msg, None, None)?;
        self.execute(
            &ExecuteMsg::AddModules {
                modules: vec![(
                    ModuleInfo::from_id(
                        &extension.id,
                        ModuleVersion::Version(new_version.to_string()),
                    )?,
                    ModuleReference::Extension(extension.address()?),
                )],
            },
            None,
        )?;
        Ok(())
    }

    pub fn add_code_ids(&self, version: Version) -> anyhow::Result<()> {
        let code_ids = self.get_chain().state().get_all_code_ids()?;
        let _addresses = self.get_chain().state().get_all_addresses()?;
        let mut modules = vec![];
        for app in registry::CORE {
            let code_id = code_ids.get(*app).unwrap();
            modules.push((
                ModuleInfo::from_id(app, ModuleVersion::Version(version.to_string()))?,
                ModuleReference::App(*code_id),
            ))
        }
        // for app in registry::APPS {
        //     let code_id = code_ids.get(app.clone()).unwrap();
        //     modules.push((ModuleInfo::from_id(app, ModuleVersion::Version(version.to_string()))?,ModuleReference::App(code_id.clone())))
        // }
        // for extension in registry::EXTENSION_CONTRACTS {
        //     let address = addresses.get(extension.clone()).unwrap();
        //     modules.push((ModuleInfo::from_id(&extension, ModuleVersion::Version(version.to_string()))?,ModuleReference::Extension(address.clone())))
        // }
        self.execute(&ExecuteMsg::AddModules { modules }, None)?;
        Ok(())
    }

    pub fn get_os_core(&self, os_id: u32) -> Result<Core, BootError> {
        let resp: OsCoreResponse = self.query(&QueryMsg::OsCore { os_id })?;
        Ok(resp.os_core)
    }
}

impl VersionControl<Daemon> {
    // pub fn update_code_ids(&self, new_version: Version) -> anyhow::Result<()> {
    //     let code_ids = self.chain().state().get_all_code_ids()?;
    //     for (contract_id, code_id) in code_ids {
    //         if NATIVE_CONTRACTS.contains(&contract_id.as_str()) {
    //             continue;
    //         }

    //         // Get latest code id
    //         let resp: Result<QueryCodeIdResponse, BootError> = self.query(&QueryMsg::CodeId {
    //             module: ModuleInfo {
    //                 name: contract_id.clone(),
    //                 version: None,
    //             },
    //         });
    //         log::debug!("{:?}", resp);
    //         if new_version.pre.is_empty() {
    //             match resp {
    //                 Ok(resp) => {
    //                     let registered_code_id = resp.code_id.u64();
    //                     // If equal, continue
    //                     if registered_code_id == code_id {
    //                         continue;
    //                     } else {
    //                         let latest_version = resp.info.version;
    //                         version = latest_version.parse().unwrap();
    //                         // bump patch
    //                         version.patch += 1;
    //                     }
    //                 }
    //                 Err(_) => (),
    //             };
    //         }

    //         self.execute(
    //             &ExecuteMsg::AddCodeId {
    //                 module: contract_id.to_string(),
    //                 version: version.to_string(),
    //                 code_id,
    //             },
    //             None,
    //         )?;
    //     }
    //     Ok(())
    // }

    // pub fn update_extensions(&self) -> anyhow::Result<()> {
    //     for contract_name in chain_state.keys() {
    //         if !EXTENSION_CONTRACTS.contains(&contract_name.as_str()) {
    //             continue;
    //         }

    //         // Get local addr
    //         let address: String = chain_state[contract_name].as_str().unwrap().into();

    //         // Get latest addr
    //         let resp: Result<QueryExtensionAddressResponse, BootError> =
    //             self.query(&QueryMsg::ExtensionAddress {
    //                 module: ModuleInfo {
    //                     name: contract_name.clone(),
    //                     version: None,
    //                 },
    //             });
    //         log::debug!("{:?}", resp);
    //         let mut version = self.deployment_version.clone();
    //         match resp {
    //             Ok(resp) => {
    //                 let registered_addr = resp.address.to_string();

    //                 // If equal, continue
    //                 if registered_addr == address {
    //                     continue;
    //                 } else {
    //                     let latest_version = resp.info.version;
    //                     version = latest_version.parse().unwrap();
    //                     // bump patch
    //                     version.patch += 1;
    //                 }
    //             }
    //             Err(_) => (),
    //         };

    //         self.execute(
    //             &ExecuteMsg::AddExtension {
    //                 module: contract_name.to_string(),
    //                 version: version.to_string(),
    //                 address,
    //             },
    //             None,
    //         )?;
    //     }
    //     Ok(())
    // }
}
