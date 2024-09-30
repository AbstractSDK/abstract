use abstract_sdk::{
    std::IBC_CLIENT,
    std::{
        module_factory::FactoryModuleInstallConfig,
        objects::{
            module::ModuleInfo, module_reference::ModuleReference,
            registry::RegistryContract,
        },
    },
    *,
};
use abstract_std::objects::module;
use cosmwasm_std::{
    from_json, to_json_binary, Addr, BankMsg, Binary, CanonicalAddr, Coin, Coins, CosmosMsg, Deps,
    DepsMut, Env, MessageInfo, WasmMsg,
};
use feature_objects::AnsHost;
use serde_cw_value::Value;

use crate::{
    contract::{ModuleFactoryResponse, ModuleFactoryResult},
    error::ModuleFactoryError,
    state::*,
};

/// Function that starts the creation of the Modules
pub fn execute_create_modules(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    modules: Vec<FactoryModuleInstallConfig>,
    salt: Binary,
) -> ModuleFactoryResult {
    let block_height = env.block.height;
    // Verify sender is active Account manager
    // Construct feature object to access registry functions
    let registry = RegistryContract::new(deps.api)?;
    let ans_host = AnsHost::new(deps.api)?;

    // assert that sender is manager
    let account_base = registry.assert_account(&info.sender, &deps.querier)?;

    // get module info and module config for further use
    let (infos, init_msgs): (Vec<ModuleInfo>, Vec<Option<Binary>>) =
        modules.into_iter().map(|m| (m.module, m.init_msg)).unzip();

    let modules_responses = registry.query_modules_configs(infos, &deps.querier)?;

    // fees
    let mut fee_msgs = vec![];
    let mut sum_of_monetization = Coins::default();

    // install messages
    let mut module_instantiate_messages = Vec::with_capacity(modules_responses.len());

    // Register modules on manager
    let mut modules_to_register: Vec<Addr> = vec![];

    // Attributes logging
    let mut module_ids: Vec<String> = Vec::with_capacity(modules_responses.len());

    let mut at_least_one_standalone = false;

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
                let namespace_account = registry
                    .query_namespace(new_module.info.namespace.clone(), &deps.querier)?
                    // It's safe to assume this namespace is claimed because
                    // modules gets unregistered when namespace is unclaimed
                    .unwrap();
                fee_msgs.push(CosmosMsg::Bank(BankMsg::Send {
                    to_address: namespace_account.account_base.addr().to_string(),
                    amount: vec![fee],
                }));
            }
            abstract_std::objects::module::Monetization::None => {}
            // The monetization must be known to the factory for a module to be installed
            _ => return Err(ModuleFactoryError::ModuleNotInstallable {}),
        };

        for init_coin in new_module_init_funds.clone() {
            sum_of_monetization.add(init_coin)?;
        }

        match &new_module.reference {
            ModuleReference::App(code_id) => {
                let init_msg = owner_init_msg.unwrap();
                let init_msg_as_value: Value = from_json(init_msg)?;
                // App base message
                let app_base_msg = abstract_std::app::BaseInstantiateMsg {
                    ans_host_address: ans_host.address.to_string(),
                    registry_address: registry.address.to_string(),
                    account_base: account_base.clone(),
                };

                let app_init_msg = abstract_std::app::InstantiateMsg::<Value> {
                    base: app_base_msg,
                    module: init_msg_as_value,
                };
                let (addr, init_msg) = instantiate2_contract(
                    deps.as_ref(),
                    canonical_contract_addr.clone(),
                    block_height,
                    *code_id,
                    to_json_binary(&app_init_msg)?,
                    salt.clone(),
                    Some(account_base.addr().clone()),
                    new_module_init_funds,
                    &new_module.info,
                )?;
                let module_address = deps.api.addr_humanize(&addr)?;
                modules_to_register.push(module_address);
                module_instantiate_messages.push(init_msg);
            }
            // Adapter or services is not installed but registered instead, so we don't push to the `installed_modules`
            ModuleReference::Adapter(addr) | ModuleReference::Service(addr) => {
                modules_to_register.push(addr.clone());
            }
            ModuleReference::Standalone(code_id) => {
                at_least_one_standalone = true;
                let (addr, init_msg) = instantiate2_contract(
                    deps.as_ref(),
                    canonical_contract_addr.clone(),
                    block_height,
                    *code_id,
                    owner_init_msg.unwrap(),
                    salt.clone(),
                    Some(account_base.addr().clone()),
                    new_module_init_funds,
                    &new_module.info,
                )?;
                let module_address = deps.api.addr_humanize(&addr)?;
                modules_to_register.push(module_address);
                module_instantiate_messages.push(init_msg);
            }
            ModuleReference::Native(native_address) => {
                if new_module.info.id() == IBC_CLIENT {
                    modules_to_register.push(native_address.clone());
                    continue;
                }
                return Err(ModuleFactoryError::ModuleNotInstallable {});
            }
            _ => return Err(ModuleFactoryError::ModuleNotInstallable {}),
        };
    }

    // If we have at least one standalone installed, then have to save this to state as \
    // Standalone may need this information for AccountIdentification \
    // Contract Info query does not work during instantiation on self contract, because contract does not exist yet.
    if at_least_one_standalone {
        CURRENT_BASE.save(deps.storage, &account_base)?;
    }

    let sum_of_monetization = sum_of_monetization.into_vec();
    if sum_of_monetization != info.funds {
        return Err(std::AbstractError::Fee(format!(
            "Invalid fee payment sent. Expected {:?}, sent {:?}",
            sum_of_monetization, info.funds
        ))
        .into());
    }

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

    let addr = cosmwasm_std::instantiate2_address(
        wasm_info.checksum.as_slice(),
        &creator_addr,
        salt.as_slice(),
    )?;

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

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use abstract_std::module_factory::ExecuteMsg;
    use cosmwasm_std::{
        testing::{message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage},
        to_json_binary, OwnedDeps,
    };
    use speculoos::prelude::*;

    use super::*;
    use crate::{contract::execute, test_common::*};

    type ModuleFactoryTestResult = Result<(), ModuleFactoryError>;

    fn execute_as(deps: DepsMut, sender: &Addr, msg: ExecuteMsg) -> ModuleFactoryResult {
        execute(deps, mock_env(), message_info(sender, &[]), msg)
    }

    fn test_only_admin(
        msg: ExecuteMsg,
        deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>,
    ) -> ModuleFactoryTestResult {
        let not_admin = deps.api.addr_make("not_admin");
        let res = execute(
            deps.as_mut(),
            mock_env(),
            message_info(&not_admin, &[]),
            msg,
        );
        assert_that!(&res)
            .is_err()
            .is_equal_to(ModuleFactoryError::Ownership(
                cw_ownable::OwnershipError::NotOwner {},
            ));

        Ok(())
    }

    mod update_ownership {
        use abstract_testing::prelude::AbstractMockAddrs;

        use super::*;

        #[test]
        fn only_admin() -> ModuleFactoryTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            let msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
                new_owner: "new_owner".to_string(),
                expiry: None,
            });

            test_only_admin(msg, &mut deps)
        }

        #[test]
        fn update_owner() -> ModuleFactoryTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let abstr = AbstractMockAddrs::new(deps.api);

            let new_admin = deps.api.addr_make("new_admin");
            // First update to transfer
            let transfer_msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
                new_owner: new_admin.to_string(),
                expiry: None,
            });

            let _transfer_res = execute_as(deps.as_mut(), &abstr.owner, transfer_msg)?;

            // Then update and accept as the new owner
            let accept_msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::AcceptOwnership);
            let _accept_res = execute_as(deps.as_mut(), &new_admin, accept_msg).unwrap();

            assert_that!(cw_ownable::get_ownership(&deps.storage).unwrap().owner)
                .is_some()
                .is_equal_to(new_admin);

            Ok(())
        }
    }

    mod instantiate_contract {
        use super::*;

        use abstract_std::objects::{module::ModuleVersion, AccountId};
        use cosmwasm_std::{coin, Api, Checksum, CodeInfoResponse, Empty, QuerierResult};

        #[test]
        fn should_create_msg_with_instantiate2_msg() -> ModuleFactoryTestResult {
            let mut deps = mock_dependencies();
            deps.querier.update_wasm(|request| match request {
                cosmwasm_std::WasmQuery::CodeInfo { code_id } => {
                    let deps_v2 = mock_dependencies();
                    let new_addr = deps_v2.api.addr_make("aloha");
                    let canonical = deps_v2.api.addr_canonicalize(new_addr.as_str()).unwrap();
                    let creator = mock_dependencies().api.addr_humanize(&canonical).unwrap();
                    QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(
                        to_json_binary(&CodeInfoResponse::new(
                            *code_id,
                            creator.clone(),
                            Checksum::from_hex(
                                "13a1fc994cc6d1c81b746ee0c0ff6f90043875e0bf1d9be6b7d779fc978dc2a5",
                            )
                            .unwrap(),
                        ))
                        .unwrap(),
                    ))
                }
                _ => panic!("handling only code_info"),
            });

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
}
