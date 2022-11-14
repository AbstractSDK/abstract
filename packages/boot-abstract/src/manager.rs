use std::fmt::Debug;

use abstract_os::objects::module::ModuleInfo;
use abstract_os::objects::module::ModuleVersion;
use boot_core::state::StateInterface;
use cosmwasm_std::{to_binary, Addr, Binary};

use serde::Serialize;

use abstract_os::manager::*;

use crate::AbstractOS;
use boot_core::{BootError, Contract, IndexResponse, TxHandler, TxResponse};

pub type Manager<Chain> = AbstractOS<Chain, ExecuteMsg, InstantiateMsg, QueryMsg, MigrateMsg>;

impl<Chain: TxHandler + Clone> Manager<Chain>
where
    TxResponse<Chain>: IndexResponse,
{
    pub fn new(name: &str, chain: &Chain) -> Self {
        Self(
            Contract::new(name, chain).with_wasm_path("manager"), // .with_mock(Box::new(
                                                                  //     ContractWrapper::new_with_empty(
                                                                  //         ::contract::execute,
                                                                  //         ::contract::instantiate,
                                                                  //         ::contract::query,
                                                                  //     ),
                                                                  // ))
        )
    }

    // pub fn update_terraswap_trader(
    //     &self,
    //     api: &str,
    //     to_add: Option<Vec<String>>,
    //     to_remove: Option<Vec<String>>,
    // ) -> Result<(), BootError> {
    //     self.execute(
    //         &ExecuteMsg::ExecOnModule {
    //             module_id: api.into(),
    //             exec_msg: to_binary(&<ApiExecuteMsg<Empty>>::Configure(BaseExecuteMsg::UpdateTraders {
    //                 to_add,
    //                 to_remove,
    //             }))
    //             .unwrap(),
    //         },
    //         None,
    //     )?;
    //     Ok(())
    // }

    pub fn upgrade_module<M: Serialize>(
        &self,
        module_id: &str,
        migrate_msg: &M,
    ) -> Result<(), BootError> {
        self.execute(
            &ExecuteMsg::Upgrade {
                module: ModuleInfo::from_id(module_id, ModuleVersion::Latest {})?,
                migrate_msg: Some(to_binary(migrate_msg)?),
            },
            None,
        )?;
        Ok(())
    }

    pub fn install_module<M: Serialize>(
        &self,
        module_id: &str,
        init_msg: Option<&M>,
    ) -> Result<(), BootError> {
        self.execute(
            &ExecuteMsg::CreateModule {
                module: ModuleInfo::from_id(module_id, ModuleVersion::Latest {})?,
                init_msg: init_msg.map(to_binary).transpose()?,
            },
            None,
        )?;
        Ok(())
    }

    pub fn execute_on_module(&self, module: &str, msg: impl Serialize) -> Result<(), BootError> {
        self.execute(
            &ExecuteMsg::ExecOnModule {
                module_id: module.into(),
                exec_msg: to_binary(&msg).unwrap(),
            },
            None,
        )?;
        Ok(())
    }

    pub fn add_module<
        I: Serialize + Debug,
        H: Serialize + Debug,
        N: Serialize + Debug,
        S: Serialize + Debug,
    >(
        &self,
        module: &Contract<Chain, H, I, N, S>,
        init_msg: Option<&I>,
        contract_id: &str,
        version: String,
    ) -> Result<TxResponse<Chain>, BootError> {
        let mut msg: Option<Binary> = None;
        if init_msg.is_some() {
            msg = init_msg.map(|msg| to_binary(msg).unwrap());
        }
        let result = self.execute(
            &ExecuteMsg::CreateModule {
                module: ModuleInfo::from_id(contract_id, ModuleVersion::Version(version))?,
                init_msg: msg,
            },
            None,
        )?;

        let module_address = result.event_attr_value("wasm", "new module:")?;
        self.chain()
            .state()
            .set_address(&module.id, &Addr::unchecked(module_address));

        Ok(result)
    }
}

// pub fn get_module_kind(name: &str) -> anyhow::Result<ModuleKind> {
//     if [TERRASWAP].contains(&name) {
//         return Ok(ModuleKind::API);
//     } else if [LIQUIDITY_INTERFACE, SUBSCRIPTION].contains(&name) {
//         Ok(ModuleKind::AddOn)
//     } else {
//         return Err(anyhow::Error::msg(
//             "The requested module to be added is not a module",
//         ));
//     }
// }
