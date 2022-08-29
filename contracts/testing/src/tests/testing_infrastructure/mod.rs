mod common_integration;
mod instantiate;
mod module_uploader;
mod os_creation;
mod upload;
mod verify;

pub mod env {
    use std::collections::HashMap;

    pub use super::common_integration::*;
    pub use super::module_uploader::*;
    pub use super::os_creation::init_os;
    use super::os_creation::init_primary_os;
    use super::upload::upload_base_contracts;
    use abstract_os::manager::{self as ManagerMsgs, ManagerModuleInfo};
    use abstract_os::version_control::Core;
    use anyhow::Result as AnyResult;
    use cosmwasm_std::{attr, to_binary, Addr, Uint128};
    use cw_multi_test::{App, AppResponse, Executor};
    use serde::Serialize;
    pub struct AbstractEnv {
        pub native_contracts: NativeContracts,
        pub code_ids: HashMap<String, u64>,
        pub os_store: HashMap<u32, Core>,
    }

    impl AbstractEnv {
        pub fn new(app: &mut App, sender: &Addr) -> Self {
            let (code_ids, native_contracts) = upload_base_contracts(app);
            let mut os_store: HashMap<u32, Core> = HashMap::new();

            init_os(app, &sender, &native_contracts, &mut os_store).expect("created first os");

            init_primary_os(app, &sender, &native_contracts, &mut os_store).unwrap();

            app.update_block(|b| {
                b.time = b.time.plus_seconds(6);
                b.height += 1;
            });

            AbstractEnv {
                native_contracts,
                code_ids,
                os_store,
            }
        }
    }

    pub fn get_os_state(
        app: &App,
        os_store: &HashMap<u32, Core>,
        os_id: &u32,
    ) -> AnyResult<HashMap<String, Addr>> {
        let manager_addr: Addr = os_store.get(os_id).unwrap().manager.clone();
        // Check OS
        let mut resp: ManagerMsgs::QueryModuleInfosResponse = app.wrap().query_wasm_smart(
            &manager_addr,
            &ManagerMsgs::QueryMsg::ModuleInfos {
                page_token: None,
                page_size: None,
            },
        )?;
        let mut state = HashMap::new();
        while !resp.module_infos.is_empty() {
            let mut last_module: Option<String> = None;
            for ManagerModuleInfo {
                address,
                name,
                version: _,
            } in resp.module_infos
            {
                last_module = Some(name.clone());
                state.insert(name, Addr::unchecked(address));
            }
            resp = app.wrap().query_wasm_smart(
                &manager_addr,
                &ManagerMsgs::QueryMsg::ModuleInfos {
                    page_token: last_module,
                    page_size: None,
                },
            )?;
        }
        Ok(state)
    }

    pub fn exec_msg_on_manager<T: Serialize>(
        app: &mut App,
        sender: &Addr,
        manager_addr: &Addr,
        module_name: &str,
        encapsuled_msg: &T,
    ) -> AnyResult<AppResponse> {
        let msg = abstract_os::manager::ExecuteMsg::ExecOnModule {
            module_name: module_name.into(),
            exec_msg: to_binary(encapsuled_msg)?,
        };
        app.execute_contract(sender.clone(), manager_addr.clone(), &msg, &[])
    }

    /// Mint tokens
    pub fn mint_tokens(
        app: &mut App,
        owner: &Addr,
        token_instance: &Addr,
        amount: Uint128,
        to: String,
    ) {
        let msg = cw20::Cw20ExecuteMsg::Mint {
            recipient: to.clone(),
            amount,
        };
        let res = app
            .execute_contract(owner.clone(), token_instance.clone(), &msg, &[])
            .unwrap();
        assert_eq!(res.events[1].attributes[1], attr("action", "mint"));
        assert_eq!(res.events[1].attributes[2], attr("to", to));
        assert_eq!(res.events[1].attributes[3], attr("amount", amount));
    }

    pub fn token_balance(app: &App, token_instance: &Addr, owner: &Addr) -> u128 {
        let balance: cw20::BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                token_instance,
                &cw20_base::msg::QueryMsg::Balance {
                    address: owner.to_string(),
                },
            )
            .unwrap();
        balance.balance.u128()
    }
}
