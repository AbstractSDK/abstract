use abstract_core::objects::module;

use crate::contract::ModuleFactoryResponse;
use crate::{contract::ModuleFactoryResult, error::ModuleFactoryError, state::*};
use abstract_sdk::{
    core::{
        module_factory::FactoryModuleInstallConfig,
        objects::{
            module::ModuleInfo, module_reference::ModuleReference,
            version_control::VersionControlContract,
        },
    },
    *,
};
use cosmwasm_std::{
    Addr, BankMsg, Binary, CanonicalAddr, Coin, Coins, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    StdResult, WasmMsg,
};

/// Function that starts the creation of the Modules
pub fn execute_create_modules(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    modules: Vec<FactoryModuleInstallConfig>,
    salt: Binary,
) -> ModuleFactoryResult {
    let config = CONFIG.load(deps.storage)?;
    let block_height = env.block.height;
    // Verify sender is active Account manager
    // Construct feature object to access registry functions
    let binding = VersionControlContract::new(config.version_control_address);

    let version_registry = binding.module_registry(deps.as_ref());
    let account_registry = binding.account_registry(deps.as_ref());

    // assert that sender is manager
    let account_base = account_registry.assert_manager(&info.sender)?;

    // get module info and module config for further use
    let (infos, init_msgs): (Vec<ModuleInfo>, Vec<Option<Binary>>) =
        modules.into_iter().map(|m| (m.module, m.init_msg)).unzip();

    let modules_responses = version_registry.query_modules_configs(infos)?;

    // fees
    let mut fee_msgs = vec![];
    let mut sum_of_monetization = Coins::default();

    // install messages
    let mut module_instantiate_messages = Vec::with_capacity(modules_responses.len());

    // Register modules on manager
    let mut modules_to_register: Vec<Addr> = vec![];

    // Attributes logging
    let mut module_ids: Vec<String> = Vec::with_capacity(modules_responses.len());

    let canonical_contract_addr = deps.api.addr_canonicalize(env.contract.address.as_str())?;
    for (owner_init_msg, module_response) in
        init_msgs.into_iter().zip(modules_responses.into_iter())
    {
        let new_module = module_response.module;
        let new_module_monetization = module_response.config.monetization;
        let new_module_init_funds = module_response.config.instantiation_funds;
        module_ids.push(new_module.info.id_with_version());

        // We validate the fee if it was required by the version control to install this module
        match new_module_monetization {
            module::Monetization::InstallFee(f) => {
                let fee = f.fee();
                sum_of_monetization.add(fee.clone())?;
                // We transfer that fee to the namespace owner if there is
                let namespace_account =
                    version_registry.query_namespace(new_module.info.namespace.clone())?;
                fee_msgs.push(CosmosMsg::Bank(BankMsg::Send {
                    to_address: namespace_account.account_base.proxy.to_string(),
                    amount: vec![fee],
                }));
            }
            abstract_core::objects::module::Monetization::None => {}
            // The monetization must be known to the factory for a module to be installed
            _ => return Err(ModuleFactoryError::ModuleNotInstallable {}),
        };

        for init_coin in new_module_init_funds.clone() {
            sum_of_monetization.add(init_coin)?;
        }

        match &new_module.reference {
            ModuleReference::App(code_id) => {
                let (addr, init_msg) = instantiate2_contract(
                    deps.as_ref(),
                    canonical_contract_addr.clone(),
                    block_height,
                    *code_id,
                    owner_init_msg.unwrap(),
                    salt.clone(),
                    Some(account_base.manager.clone()),
                    new_module_init_funds,
                    &new_module.info,
                )?;
                let module_address = deps.api.addr_humanize(&addr)?;
                modules_to_register.push(module_address);
                module_instantiate_messages.push(init_msg);
            }
            // Adapter is not installed but registered instead, so we don't push to the `installed_modules`
            ModuleReference::Adapter(addr) => {
                modules_to_register.push(addr.clone());
            }
            ModuleReference::Standalone(code_id) => {
                let (addr, init_msg) = instantiate2_contract(
                    deps.as_ref(),
                    canonical_contract_addr.clone(),
                    block_height,
                    *code_id,
                    owner_init_msg.unwrap(),
                    salt.clone(),
                    Some(account_base.manager.clone()),
                    new_module_init_funds,
                    &new_module.info,
                )?;
                let module_address = deps.api.addr_humanize(&addr)?;
                modules_to_register.push(module_address);
                module_instantiate_messages.push(init_msg);
            }
            _ => return Err(ModuleFactoryError::ModuleNotInstallable {}),
        };
    }

    let sum_of_monetization = sum_of_monetization.into_vec();
    if sum_of_monetization != info.funds {
        return Err(core::AbstractError::Fee(format!(
            "Invalid fee payment sent. Expected {:?}, sent {:?}",
            sum_of_monetization, info.funds
        ))
        .into());
    }

    let context = Context { account_base };
    CONTEXT.save(deps.storage, &context)?;

    let new_modules = new_module_addrs(&modules_to_register)?;

    let response = ModuleFactoryResponse::new(
        "create_modules",
        [
            ("module_ids", format!("{module_ids:?}")),
            ("new_modules", new_modules),
        ],
    )
    .add_messages(fee_msgs)
    .add_messages(module_instantiate_messages);

    Ok(response)
}

#[allow(clippy::too_many_arguments)]
fn instantiate2_contract(
    deps: Deps,
    creator_addr: CanonicalAddr,
    block_height: u64,
    code_id: u64,
    init_msg: Binary,
    salt: Binary,
    admin: Option<Addr>,
    funds: Vec<Coin>,
    module_info: &ModuleInfo,
) -> ModuleFactoryResult<(CanonicalAddr, CosmosMsg)> {
    let wasm_info = deps.querier.query_wasm_code_info(code_id)?;

    let addr =
        cosmwasm_std::instantiate2_address(&wasm_info.checksum, &creator_addr, salt.as_slice())?;

    Ok((
        addr,
        WasmMsg::Instantiate2 {
            code_id,
            funds,
            admin: admin.map(Into::into),
            label: format!("Module: {module_info}, Height {block_height}"),
            msg: init_msg,
            salt,
        }
        .into(),
    ))
}

pub fn new_module_addrs(modules_to_register: &[Addr]) -> ModuleFactoryResult<String> {
    let module_addrs = modules_to_register
        .iter()
        .map(|addr| addr.as_str())
        .collect::<Vec<&str>>()
        .join(",");

    Ok(module_addrs)
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
        use abstract_core::objects::{module::ModuleVersion, AccountId};
        use cosmwasm_std::{
            coin, testing::mock_info, to_json_binary, Api, CodeInfoResponse, Empty, HexBinary,
            QuerierResult,
        };

        #[test]
        fn should_create_msg_with_instantiate2_msg() -> ModuleFactoryTestResult {
            let mut deps = mock_dependencies();
            deps.querier.update_wasm(|request| match request {
                cosmwasm_std::WasmQuery::CodeInfo { code_id } => {
                    let deps_v2 = mock_dependencies();
                    let new_addr = deps_v2.api.addr_make("aloha");
                    let canonical = deps_v2.api.addr_canonicalize(new_addr.as_str()).unwrap();
                    let creator = mock_dependencies()
                        .api
                        .addr_humanize(&canonical)
                        .unwrap()
                        .into_string();
                    QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(
                        to_json_binary(&CodeInfoResponse::new(
                            *code_id,
                            creator.clone(),
                            HexBinary::from_hex(
                                "13a1fc994cc6d1c81b746ee0c0ff6f90043875e0bf1d9be6b7d779fc978dc2a5",
                            )
                            .unwrap(),
                        ))
                        .unwrap(),
                    ))
                }
                _ => panic!("handling only code_info"),
            });
            let _info = mock_info("anyone", &[]);

            let expected_module_init_msg = to_json_binary(&Empty {}).unwrap();
            let expected_code_id = 10u64;

            let expected_module_info =
                ModuleInfo::from_id("test:module", ModuleVersion::Version("1.2.3".to_string()))
                    .unwrap();

            let some_block_height = 500u64;
            let contract_addr = deps.api.addr_make("contract");
            let creator_addr = deps.api.addr_canonicalize(contract_addr.as_str()).unwrap();
            let account_id = AccountId::local(1);
            let mut salt_bytes: Vec<u8> = Vec::with_capacity(32);
            salt_bytes.extend(some_block_height.to_be_bytes());
            salt_bytes.extend(account_id.seq().to_be_bytes());
            salt_bytes.extend(
                account_id
                    .trace()
                    .to_string()
                    .into_bytes()
                    .into_iter()
                    .take(20)
                    .collect::<Vec<u8>>(),
            );
            let salt = Binary::from(salt_bytes);

            let actual = instantiate2_contract(
                deps.as_ref(),
                creator_addr,
                some_block_height,
                expected_code_id,
                expected_module_init_msg.clone(),
                salt.clone(),
                None,
                vec![coin(5, "ucosm")],
                &expected_module_info,
            );

            let expected_init_msg = WasmMsg::Instantiate2 {
                code_id: expected_code_id,
                funds: vec![coin(5, "ucosm")],
                admin: None,
                label: format!("Module: {expected_module_info}, Height {some_block_height}"),
                msg: expected_module_init_msg,
                salt,
            };

            assert_that!(actual).is_ok();

            let (_addr, actual_msg) = actual.unwrap();

            let actual_init_msg: CosmosMsg = actual_msg;

            assert_that!(actual_init_msg).matches(|i| matches!(i, CosmosMsg::Wasm { .. }));
            assert_that!(actual_init_msg).is_equal_to(CosmosMsg::from(expected_init_msg));

            Ok(())
        }
    }

    use cosmwasm_std::to_json_binary;

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
                to_json_binary(&"tasty pizza usually has pineapple").unwrap(),
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
