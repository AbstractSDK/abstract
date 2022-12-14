use boot_core::{
    interface::{BootExecute, BootQuery, ContractInstance},
    prelude::boot_contract,
    BootEnvironment, BootError, Contract, Daemon, IndexResponse, TxResponse,
};
use cosmwasm_std::Addr;
use semver::Version;

use abstract_sdk::os::{
    objects::{
        module::{ModuleInfo, ModuleVersion},
        module_reference::ModuleReference,
    },
    version_control::*,
    VERSION_CONTROL,
};

use crate::deployment::{self, OS};

#[boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct VersionControl<Chain>;

impl<Chain: BootEnvironment> VersionControl<Chain>
where
    TxResponse<Chain>: IndexResponse,
{
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

    pub fn register_core(&self, os: &OS<Chain>, version: &str) -> Result<(), BootError> {
        let manager = os.manager.as_instance();
        self.execute(
            &ExecuteMsg::AddModules {
                modules: vec![(
                    ModuleInfo::from_id(&manager.id, ModuleVersion::Version(version.to_string()))?,
                    ModuleReference::Core(manager.code_id()?),
                )],
            },
            None,
        )?;
        log::info!("Module {} registered", manager.id);

        let proxy = os.proxy.as_instance();
        self.execute(
            &ExecuteMsg::AddModules {
                modules: vec![(
                    ModuleInfo::from_id(&proxy.id, ModuleVersion::Version(version.to_string()))?,
                    ModuleReference::Core(proxy.code_id()?),
                )],
            },
            None,
        )?;
        log::info!("Module {} registered", proxy.id);
        Ok(())
    }

    pub fn register_apps(
        &self,
        modules: Vec<&Contract<Chain>>,
        version: &Version,
    ) -> Result<(), BootError> {
        let apps_to_register: Result<Vec<(ModuleInfo, ModuleReference)>, BootError> = modules
            .iter()
            .map(|app| {
                Ok((
                    ModuleInfo::from_id(&app.id, ModuleVersion::Version(version.to_string()))?,
                    ModuleReference::App(app.code_id()?),
                ))
            })
            .collect();
        self.execute(
            &ExecuteMsg::AddModules {
                modules: apps_to_register?,
            },
            None,
        )?;
        Ok(())
    }

    pub fn register_extensions(
        &self,
        extensions: Vec<&Contract<Chain>>,
        version: &Version,
    ) -> Result<(), BootError> {
        let extensions_to_register: Result<Vec<(ModuleInfo, ModuleReference)>, BootError> =
            extensions
                .iter()
                .map(|ex| {
                    Ok((
                        ModuleInfo::from_id(&ex.id, ModuleVersion::Version(version.to_string()))?,
                        ModuleReference::Extension(ex.address()?),
                    ))
                })
                .collect();
        self.execute(
            &ExecuteMsg::AddModules {
                modules: extensions_to_register?,
            },
            None,
        )?;
        Ok(())
    }

    pub fn register_native(
        &self,
        deployment: &deployment::Deployment<Chain>,
    ) -> Result<(), BootError> {
        let modules: Result<Vec<(ModuleInfo, ModuleReference)>, BootError> = deployment
            .contracts()
            .iter()
            .map(|contr| {
                Ok((
                    ModuleInfo::from_id(
                        &contr.id,
                        ModuleVersion::Version(deployment.version.to_string()),
                    )?,
                    ModuleReference::Native(contr.address()?),
                ))
            })
            .collect();

        self.execute(&ExecuteMsg::AddModules { modules: modules? }, None)?;
        Ok(())
    }

    pub fn get_os_core(&self, os_id: u32) -> Result<Core, BootError> {
        let resp: OsCoreResponse = self.query(&QueryMsg::OsCore { os_id })?;
        Ok(resp.os_core)
    }
}

impl VersionControl<Daemon> {
    // pub fn update_code_ids(&self, new_version: Version) -> anyhow::Result<()> {
    //     let code_ids = self.get_chain().state().get_all_code_ids()?;
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
