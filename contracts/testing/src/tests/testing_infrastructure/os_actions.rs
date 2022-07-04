use abstract_os::manager as manager_msg;
use abstract_os::version_control::Core;
use abstract_os::{modules::Module};
use anyhow::Result as AnyResult;
use cosmwasm_std::{to_binary, Addr};
use cw_multi_test::{App, Executor};
use serde::Serialize;

pub trait CoreActions {
    fn add_module<T: Serialize>(
        &self,
        app: &mut App,
        sender: &Addr,
        module: Module,
        initmsg: Option<T>,
    ) -> AnyResult<()>;
}

impl CoreActions for Core {
    fn add_module<T: Serialize>(
        &self,
        app: &mut App,
        sender: &Addr,
        module: Module,
        init_msg: Option<T>,
    ) -> AnyResult<()> {
        let msg = manager_msg::ExecuteMsg::CreateModule {
            module,
            init_msg: init_msg.map(|msg| to_binary(&msg).unwrap()),
        };
        app.execute_contract(sender.clone(), self.manager.clone(), &msg, &[])?;
        Ok(())
    }
}
