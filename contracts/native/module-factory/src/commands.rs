use abstract_core::objects::module;

use crate::contract::ModuleFactoryResponse;
use crate::{
    contract::ModuleFactoryResult, error::ModuleFactoryError,
    response::MsgInstantiateContractResponse, state::*,
};
use abstract_sdk::{
    core::{
        manager::ExecuteMsg as ManagerMsg,
        objects::{module::ModuleInfo, module_reference::ModuleReference},
    },
    feature_objects::VersionControlContract,
    *,
};
use cosmwasm_std::{
    wasm_execute, Addr, Binary, CosmosMsg, DepsMut, Empty, Env, MessageInfo, ReplyOn, StdError,
    StdResult, SubMsg, SubMsgResult, WasmMsg,
};
use protobuf::Message;

pub const CREATE_APP_RESPONSE_ID: u64 = 1u64;
pub const CREATE_STANDALONE_RESPONSE_ID: u64 = 4u64;

/// Function that starts the creation of the Account
pub fn execute_create_module(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module_info: ModuleInfo,
    owner_init_msg: Option<Binary>,
) -> ModuleFactoryResult {
    let config = CONFIG.load(deps.storage)?;
    // Verify sender is active Account manager
    // Construct feature object to access registry functions
    let binding = VersionControlContract::new(config.version_control_address);

    let version_registry = binding.module_registry(deps.as_ref());
    let account_registry = binding.account_registry(deps.as_ref());
    // assert that sender is manager
    let account_base = account_registry.assert_manager(&info.sender)?;

    let new_module = version_registry.query_module(module_info)?;

    // TODO: check if this can be generalized for some contracts
    // aka have default values for each kind of module that only get overwritten if a specific init_msg is saved.
    // let fixed_binary = MODULE_INIT_BINARIES.may_load(deps.storage, new_module.info.clone())?;
    // let init_msg = ModuleInitMsg {
    //     fixed_init: fixed_binary,
    //     owner_init: owner_init_msg,
    // }
    // .format()?;

    // Set context for after init
    CONTEXT.save(
        deps.storage,
        &Context {
            account_base: Some(account_base.clone()),
            module: Some(new_module.clone()),
        },
    )?;
    let block_height = env.block.height;
    match &new_module.reference {
        ModuleReference::App(code_id) => instantiate_contract(
            block_height,
            *code_id,
            owner_init_msg.unwrap(),
            Some(account_base.manager),
            CREATE_APP_RESPONSE_ID,
            new_module.info,
        ),
        ModuleReference::Adapter(addr) => {
            let module_id = new_module.info.id_with_version();
            let register_msg: CosmosMsg<Empty> = wasm_execute(
                account_base.manager.into_string(),
                &ManagerMsg::RegisterModule {
                    module_addr: addr.to_string(),
                    module: new_module,
                },
                vec![],
            )?
            .into();
            Ok(
                ModuleFactoryResponse::new("execute_create_module", vec![("module", &module_id)])
                    .add_message(register_msg),
            )
        }
        ModuleReference::Standalone(code_id) => instantiate_contract(
            block_height,
            *code_id,
            owner_init_msg.unwrap(),
            Some(account_base.manager),
            CREATE_STANDALONE_RESPONSE_ID,
            new_module.info,
        ),
        _ => Err(ModuleFactoryError::ModuleNotInstallable {}),
    }
}

fn instantiate_contract(
    block_height: u64,
    code_id: u64,
    init_msg: Binary,
    admin: Option<Addr>,
    reply_id: u64,
    module_info: ModuleInfo,
) -> ModuleFactoryResult {
    let response = ModuleFactoryResponse::new(
        "execute_create_module",
        vec![("module", module_info.id_with_version())],
    );
    Ok(response.add_submessage(SubMsg {
        id: reply_id,
        gas_limit: None,
        msg: WasmMsg::Instantiate {
            code_id,
            funds: vec![],
            admin: admin.map(Into::into),
            label: format!("Module: {module_info}, Height {block_height}"),
            msg: init_msg,
        }
        .into(),
        reply_on: ReplyOn::Success,
    }))
}

pub fn register_contract(deps: DepsMut, result: SubMsgResult) -> ModuleFactoryResult {
    let context: Context = CONTEXT.load(deps.storage)?;
    let module = context.module.unwrap();

    // Get address of the new contract
    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(result.unwrap().data.unwrap().as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;
    let module_address = deps.api.addr_validate(res.get_contract_address())?;
    // assert the data after instantiation.
    module::assert_module_data_validity(&deps.querier, &module, Some(module_address.clone()))?;

    let register_msg: CosmosMsg<Empty> = wasm_execute(
        context.account_base.unwrap().manager.into_string(),
        &ManagerMsg::RegisterModule {
            module_addr: module_address.to_string(),
            module,
        },
        vec![],
    )?
    .into();

    clear_context(deps)?;

    Ok(
        ModuleFactoryResponse::new("register_contract", vec![("new_module", module_address)])
            .add_message(register_msg),
    )
}

// Only owner can execute it
pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    ans_host_address: Option<String>,
    version_control_address: Option<String>,
) -> ModuleFactoryResult {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut config: Config = CONFIG.load(deps.storage)?;

    if let Some(ans_host_address) = ans_host_address {
        // validate address format
        config.ans_host_address = deps.api.addr_validate(&ans_host_address)?;
    }

    if let Some(version_control_address) = version_control_address {
        // validate address format
        config.version_control_address = deps.api.addr_validate(&version_control_address)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(ModuleFactoryResponse::action("update_config"))
}

// Only owner can execute it
pub fn update_factory_binaries(
    deps: DepsMut,
    info: MessageInfo,
    to_add: Vec<(ModuleInfo, Binary)>,
    to_remove: Vec<ModuleInfo>,
) -> ModuleFactoryResult {
    // Only Admin can call this method
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    for (key, binary) in to_add.into_iter() {
        // Update function for new or existing keys
        key.assert_version_variant()?;
        let insert = |_| -> StdResult<Binary> { Ok(binary) };
        MODULE_INIT_BINARIES.update(deps.storage, &key, insert)?;
    }

    for key in to_remove {
        key.assert_version_variant()?;
        MODULE_INIT_BINARIES.remove(deps.storage, &key);
    }
    Ok(ModuleFactoryResponse::action("update_factory_binaries"))
}

fn clear_context(deps: DepsMut) -> Result<(), StdError> {
    // Set context for after init
    CONTEXT.save(
        deps.storage,
        &Context {
            account_base: None,
            module: None,
        },
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use speculoos::prelude::*;

    use crate::contract::execute;
    use crate::test_common::*;
    use abstract_core::module_factory::ExecuteMsg;

    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    type ModuleFactoryTestResult = Result<(), ModuleFactoryError>;

    fn execute_as(deps: DepsMut, sender: &str, msg: ExecuteMsg) -> ModuleFactoryResult {
        execute(deps, mock_env(), mock_info(sender, &[]), msg)
    }

    fn execute_as_admin(deps: DepsMut, msg: ExecuteMsg) -> ModuleFactoryResult {
        execute_as(deps, "admin", msg)
    }

    fn test_only_admin(msg: ExecuteMsg) -> ModuleFactoryTestResult {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut())?;

        let res = execute(deps.as_mut(), mock_env(), mock_info("not_admin", &[]), msg);
        assert_that!(&res)
            .is_err()
            .is_equal_to(ModuleFactoryError::Ownership(
                cw_ownable::OwnershipError::NotOwner {},
            ));

        Ok(())
    }

    mod update_ownership {
        use super::*;

        #[test]
        fn only_admin() -> ModuleFactoryTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
                new_owner: "new_owner".to_string(),
                expiry: None,
            });

            test_only_admin(msg)
        }

        #[test]
        fn update_owner() -> ModuleFactoryTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let new_admin = "new_admin";
            // First update to transfer
            let transfer_msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
                new_owner: new_admin.to_string(),
                expiry: None,
            });

            let _transfer_res = execute_as_admin(deps.as_mut(), transfer_msg)?;

            // Then update and accept as the new owner
            let accept_msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::AcceptOwnership);
            let _accept_res = execute_as(deps.as_mut(), new_admin, accept_msg).unwrap();

            assert_that!(cw_ownable::get_ownership(&deps.storage).unwrap().owner)
                .is_some()
                .is_equal_to(cosmwasm_std::Addr::unchecked(new_admin));

            Ok(())
        }
    }

    mod instantiate_contract {
        use super::*;
        use abstract_core::objects::module::ModuleVersion;
        use cosmwasm_std::{testing::mock_info, to_binary};

        #[test]
        fn should_create_submsg_with_instantiate_msg() -> ModuleFactoryTestResult {
            let _deps = mock_dependencies();
            let _info = mock_info("anyone", &[]);

            let expected_module_init_msg = to_binary(&Empty {}).unwrap();
            let expected_code_id = 10;
            let expected_reply_id = 69;

            let expected_module_info =
                ModuleInfo::from_id("test:module", ModuleVersion::Version("1.2.3".to_string()))
                    .unwrap();

            let some_block_height = 500;
            let actual = instantiate_contract(
                some_block_height,
                expected_code_id,
                expected_module_init_msg.clone(),
                None,
                expected_reply_id,
                expected_module_info.clone(),
            );

            let expected_init_msg = WasmMsg::Instantiate {
                code_id: expected_code_id,
                funds: vec![],
                admin: None,
                label: format!("Module: {expected_module_info}, Height {some_block_height}"),
                msg: expected_module_init_msg,
            };

            assert_that!(actual).is_ok();

            let actual_response = actual.unwrap();

            assert_that!(actual_response.messages).has_length(1);
            let actual_submsg = actual_response.messages[0].clone();

            assert_that!(actual_submsg.id).is_equal_to(expected_reply_id);
            assert_that!(actual_submsg.gas_limit).is_equal_to(None);
            assert_that!(actual_submsg.reply_on).is_equal_to(ReplyOn::Success);

            let actual_init_msg: CosmosMsg = actual_submsg.msg;

            assert_that!(actual_init_msg).matches(|i| matches!(i, CosmosMsg::Wasm { .. }));
            assert_that!(actual_init_msg).is_equal_to(CosmosMsg::from(expected_init_msg));

            Ok(())
        }
    }

    use cosmwasm_std::to_binary;

    mod update_factory_binaries {
        use super::*;
        use abstract_core::{objects::module::ModuleVersion, AbstractError};
        use abstract_testing::map_tester::*;
        use abstract_testing::prelude::TEST_ADMIN;

        fn update_module_msgs_builder(
            to_add: Vec<(ModuleInfo, Binary)>,
            to_remove: Vec<ModuleInfo>,
        ) -> ExecuteMsg {
            ExecuteMsg::UpdateFactoryBinaryMsgs { to_add, to_remove }
        }

        fn mock_entry() -> (ModuleInfo, Binary) {
            (
                ModuleInfo::from_id("test:module", ModuleVersion::Version("0.1.2".to_string()))
                    .unwrap(),
                to_binary(&"tasty pizza usually has pineapple").unwrap(),
            )
        }

        fn setup_map_tester<'a>() -> CwMapTester<
            'a,
            ExecuteMsg,
            ModuleFactoryError,
            &'a ModuleInfo,
            Binary,
            ModuleInfo,
            Binary,
        > {
            let info = mock_info(TEST_ADMIN, &[]);

            let tester = CwMapTesterBuilder::default()
                .info(info)
                .map(MODULE_INIT_BINARIES)
                .execute(execute)
                .msg_builder(update_module_msgs_builder)
                .mock_entry_builder(mock_entry)
                .from_checked_entry(|(k, v)| (k, v))
                .build()
                .unwrap();

            tester
        }

        #[test]
        fn only_admin() -> ModuleFactoryTestResult {
            let msg = ExecuteMsg::UpdateFactoryBinaryMsgs {
                to_add: vec![],
                to_remove: vec![],
            };

            test_only_admin(msg)
        }

        #[test]
        fn test_map() -> ModuleFactoryTestResult {
            let mut deps = mock_dependencies();

            mock_init(deps.as_mut()).unwrap();

            let mut map_tester = setup_map_tester();
            map_tester.test_all(&mut deps)
        }

        #[test]
        fn should_reject_latest_versions() -> ModuleFactoryTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let mut map_tester = setup_map_tester();

            let bad_entry = (
                ModuleInfo::from_id("test:module", ModuleVersion::Latest).unwrap(),
                Binary::default(),
            );

            let res = map_tester.execute_update(deps.as_mut(), (vec![bad_entry], vec![]));

            assert_that!(res)
                .is_err()
                .is_equal_to(ModuleFactoryError::Abstract(AbstractError::Assert(
                    "Module version must be set to a specific version".into(),
                )));

            Ok(())
        }
    }

    mod update_config {
        use super::*;

        #[test]
        fn only_admin() -> ModuleFactoryTestResult {
            let msg = ExecuteMsg::UpdateConfig {
                ans_host_address: None,
                version_control_address: None,
            };

            test_only_admin(msg)
        }

        #[test]
        fn update_ans_host_address() -> ModuleFactoryTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let new_ans_host = "new_ans_host";
            let msg = ExecuteMsg::UpdateConfig {
                ans_host_address: Some(new_ans_host.to_string()),
                version_control_address: None,
            };

            execute_as_admin(deps.as_mut(), msg)?;

            assert_that!(CONFIG.load(&deps.storage)?.ans_host_address)
                .is_equal_to(Addr::unchecked(new_ans_host));

            Ok(())
        }

        #[test]
        fn update_version_control_address() -> ModuleFactoryTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let new_vc = "new_version_control";
            let msg = ExecuteMsg::UpdateConfig {
                ans_host_address: None,
                version_control_address: Some(new_vc.to_string()),
            };

            execute_as_admin(deps.as_mut(), msg)?;

            assert_that!(CONFIG.load(&deps.storage)?.version_control_address)
                .is_equal_to(Addr::unchecked(new_vc));

            Ok(())
        }
    }
}
