use abstract_core::module_factory::{ModuleInstallConfig, SimulateInstallModulesResponse};
use abstract_core::objects::account::AccountTrace;
use abstract_core::objects::price_source::UncheckedPriceSource;
use abstract_core::objects::{AccountId, AssetEntry, ABSTRACT_ACCOUNT_ID};
use abstract_core::AbstractError;

use abstract_core::{manager::ExecuteMsg, objects::module::assert_module_data_validity};
use cosmwasm_std::{
    to_binary, wasm_execute, Addr, Coins, CosmosMsg, DepsMut, Empty, Env, MessageInfo,
    QuerierWrapper, ReplyOn, StdError, SubMsg, SubMsgResult, WasmMsg,
};
use protobuf::Message;

use abstract_sdk::{
    core::{
        manager::{InstantiateMsg as ManagerInstantiateMsg, InternalConfigAction},
        objects::{
            gov_type::GovernanceDetails, module::Module, module::ModuleInfo,
            module_reference::ModuleReference,
        },
        proxy::{ExecuteMsg as ProxyExecMsg, InstantiateMsg as ProxyInstantiateMsg},
        version_control::{
            AccountBase, ExecuteMsg as VCExecuteMsg, ModulesResponse, QueryMsg as VCQuery,
        },
        AbstractResult, MANAGER, PROXY,
    },
    cw_helpers::wasm_smart_query,
};

use crate::contract::AccountFactoryResponse;
use crate::{
    contract::AccountFactoryResult, error::AccountFactoryError,
    response::MsgInstantiateContractResponse, state::*,
};

pub const CREATE_ACCOUNT_PROXY_MSG_ID: u64 = 1u64;
pub const CREATE_ACCOUNT_MANAGER_MSG_ID: u64 = 2u64;

/// Function that starts the creation of the Account
#[allow(clippy::too_many_arguments)]
pub fn execute_create_account(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    governance: GovernanceDetails<String>,
    name: String,
    description: Option<String>,
    link: Option<String>,
    namespace: Option<String>,
    base_asset: Option<AssetEntry>,
    install_modules: Vec<ModuleInstallConfig>,
) -> AccountFactoryResult {
    let config = CONFIG.load(deps.storage)?;

    let governance = governance.verify(deps.as_ref(), config.version_control_contract.clone())?;

    // We want to create a local account
    // and set the origin to local
    let origin = AccountTrace::Local;

    // load the next account id
    // if it doesn't exist then it's the first account so set it to 0.
    let next_sequence = LOCAL_ACCOUNT_SEQUENCE.may_load(deps.storage)?.unwrap_or(0);

    // Check if the caller is the owner when instantiating the abstract account
    if next_sequence == ABSTRACT_ACCOUNT_ID.seq() {
        cw_ownable::assert_owner(deps.storage, &info.sender)?;
    }
    let account_id = AccountId::new(next_sequence, origin)?;

    // Query version_control for code_id of Manager contract
    let module: Module = query_module(&deps.querier, &config.version_control_contract, PROXY)?;

    let simulate_resp: SimulateInstallModulesResponse = deps.querier.query_wasm_smart(
        config.module_factory_address,
        &abstract_core::module_factory::QueryMsg::SimulateInstallModules {
            modules: install_modules.iter().map(|m| m.module.clone()).collect(),
        },
    )?;
    let funds_for_install = simulate_resp.total_required_funds;

    // Remove all funds used to install the module to pass rest to the proxy contract
    let mut funds_to_proxy = Coins::try_from(info.funds.clone()).unwrap();
    for coin in funds_for_install.clone() {
        funds_to_proxy.sub(coin).map_err(|_| {
            AbstractError::Fee(format!(
                "Invalid fee payment sent. Expected {:?}, sent {:?}",
                funds_for_install, info.funds
            ))
        })?;
    }
    // save module for after-init check
    CONTEXT.save(
        deps.storage,
        &Context {
            account_id: account_id.clone(),
            account_proxy_address: None,
            manager_module: None,
            proxy_module: Some(module.clone()),

            additional_config: AdditionalContextConfig {
                namespace,
                base_asset,
                name,
                description,
                link,
                owner: governance.into(),
            },
            install_modules,
            funds_for_install,
        },
    )?;

    if let ModuleReference::AccountBase(proxy_code_id) = module.reference {
        Ok(AccountFactoryResponse::new(
            "create_account",
            vec![
                ("account_sequence", &account_id.seq().to_string()),
                ("trace", &account_id.trace().to_string()),
            ],
        )
        // Create manager
        .add_submessage(SubMsg {
            id: CREATE_ACCOUNT_PROXY_MSG_ID,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id: proxy_code_id,
                funds: funds_to_proxy.into_vec(),
                // Currently set admin to self, update later when we know the contract's address.
                admin: Some(env.contract.address.to_string()),
                label: format!("Proxy of Account: {}", account_id),
                msg: to_binary(&ProxyInstantiateMsg {
                    account_id,
                    ans_host_address: config.ans_host_contract.to_string(),
                })?,
            }
            .into(),
            reply_on: ReplyOn::Success,
        }))
    } else {
        Err(AccountFactoryError::WrongModuleKind(
            module.info.to_string(),
            "account_base".to_string(),
        ))
    }
}

/// instantiates the Treasury contract of the newly created DAO
pub fn after_proxy_create_manager(
    deps: DepsMut,
    env: Env,
    result: SubMsgResult,
) -> AccountFactoryResult {
    let config = CONFIG.load(deps.storage)?;

    // Get address of Manager contract
    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(result.unwrap().data.unwrap().as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;
    let proxy_address = deps.api.addr_validate(res.get_contract_address())?;

    // Query version_control for code_id of proxy
    let module: Module = query_module(&deps.querier, &config.version_control_contract, MANAGER)?;

    // Update the manager address and proxy module in the context.
    let context = CONTEXT.update(deps.storage, |c| {
        Result::<_, StdError>::Ok(Context {
            account_proxy_address: Some(proxy_address.clone()),
            manager_module: Some(module.clone()),
            ..c
        })
    })?;

    if let ModuleReference::AccountBase(proxy_code_id) = module.reference {
        Ok(AccountFactoryResponse::new(
            "create_proxy",
            vec![("proxy_address", proxy_address.to_string())],
        )
        // Instantiate proxy contract
        .add_submessage(SubMsg {
            id: CREATE_ACCOUNT_MANAGER_MSG_ID,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id: proxy_code_id,
                funds: vec![],
                admin: Some(env.contract.address.into_string()),
                label: format!("Abstract Account: {}", context.account_id),
                msg: to_binary(&ManagerInstantiateMsg {
                    account_id: context.account_id,
                    version_control_address: config.version_control_contract.to_string(),
                    module_factory_address: config.module_factory_address.to_string(),
                    name: context.additional_config.name,
                    description: context.additional_config.description,
                    link: context.additional_config.link,
                    owner: context.additional_config.owner,
                    install_modules: context.install_modules,
                })?,
            }
            .into(),
            reply_on: ReplyOn::Success,
        }))
    } else {
        Err(AccountFactoryError::WrongModuleKind(
            module.info.to_string(),
            "app".to_string(),
        ))
    }
}

fn query_module(
    querier: &QuerierWrapper,
    version_control_addr: &Addr,
    module_id: &str,
) -> AbstractResult<Module> {
    let ModulesResponse { mut modules } = querier.query(&wasm_smart_query(
        version_control_addr.to_string(),
        &VCQuery::Modules {
            infos: vec![ModuleInfo::from_id_latest(module_id)?],
        },
    )?)?;

    Ok(modules.swap_remove(0).module)
}

/// Registers the Account on the version_control contract and
/// adds proxy contract address to Manager
pub fn after_proxy_add_to_manager_and_set_admin(
    deps: DepsMut,
    result: SubMsgResult,
) -> AccountFactoryResult {
    let config = CONFIG.load(deps.storage)?;
    let context = CONTEXT.load(deps.storage)?;
    CONTEXT.remove(deps.storage);

    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(result.unwrap().data.unwrap().as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;

    let manager_address = deps.api.addr_validate(res.get_contract_address())?;
    let proxy_address = context
        .account_proxy_address
        .expect("proxy address set in context");
    let account_id = context.account_id;

    // assert proxy and manager contract information is correct
    assert_module_data_validity(
        &deps.querier,
        &context
            .manager_module
            .expect("manager module set in context"),
        Some(manager_address.clone()),
    )?;
    assert_module_data_validity(
        &deps.querier,
        &context.proxy_module.expect("proxy module set in context"),
        Some(proxy_address.clone()),
    )?;

    // construct Account base
    let account_base = AccountBase {
        manager: manager_address.clone(),
        proxy: proxy_address.clone(),
    };

    // Add Account base to version_control
    let add_account_to_version_control_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.version_control_contract.to_string(),
        funds: vec![],
        msg: to_binary(&VCExecuteMsg::AddAccount {
            account_id: account_id.clone(),
            account_base,
        })?,
    });

    // add manager to whitelisted addresses
    let whitelist_manager: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: proxy_address.to_string(),
        funds: vec![],
        msg: to_binary(&ProxyExecMsg::AddModule {
            module: manager_address.to_string(),
        })?,
    });

    let set_base_asset_msg = context
        .additional_config
        .base_asset
        .map(|a| {
            Ok::<_, StdError>(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: proxy_address.to_string(),
                funds: vec![],
                msg: to_binary(&ProxyExecMsg::UpdateAssets {
                    to_add: vec![(a, UncheckedPriceSource::None)],
                    to_remove: vec![],
                })?,
            }))
        })
        .transpose()?;

    let set_namespace_msg = context
        .additional_config
        .namespace
        .map(|n| {
            Ok::<_, StdError>(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.version_control_contract.to_string(),
                funds: vec![],
                msg: to_binary(&VCExecuteMsg::ClaimNamespace {
                    account_id: account_id.clone(),
                    namespace: n,
                })?,
            }))
        })
        .transpose()?;

    let set_proxy_admin_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: proxy_address.to_string(),
        funds: vec![],
        msg: to_binary(&ProxyExecMsg::SetAdmin {
            admin: manager_address.to_string(),
        })?,
    });

    let set_wasm_admin_msgs: Vec<CosmosMsg<Empty>> = vec![
        CosmosMsg::Wasm(WasmMsg::UpdateAdmin {
            contract_addr: manager_address.to_string(),
            admin: manager_address.to_string(),
        }),
        CosmosMsg::Wasm(WasmMsg::UpdateAdmin {
            contract_addr: proxy_address.to_string(),
            admin: manager_address.to_string(),
        }),
    ];

    // Add 1 to account sequence for local origin
    if account_id.is_local() {
        LOCAL_ACCOUNT_SEQUENCE.save(deps.storage, &account_id.seq().checked_add(1).unwrap())?;
    }

    let add_proxy_address_msg = wasm_execute(
        manager_address.to_string(),
        &ExecuteMsg::UpdateInternalConfig(
            // Binary format to prevent users from easily calling the endpoint (because that's dangerous.)
            to_binary(&InternalConfigAction::UpdateModuleAddresses {
                to_add: Some(vec![(PROXY.to_string(), proxy_address.to_string())]),
                to_remove: None,
            })
            .unwrap(),
        ),
        // Attaching funds for installing modules
        context.funds_for_install,
    )?;

    // The execution order here is important.
    // Installing modules on the manager account requires that:
    // - The account is registered.
    // - The manager is the Admin of the proxy.
    // - The proxy is registered on the manager. (this last step triggers the installation of the modules.)

    let mut resp = AccountFactoryResponse::new(
        "create_manager",
        vec![("manager_address", res.get_contract_address())],
    )
    // So first register the account on the Version Control
    .add_message(add_account_to_version_control_msg)
    // Then whitelist the manager on the proxy contract so it can execute messages on behalf of the owner.
    .add_message(whitelist_manager)
    // And change the wasm-module admin to the manager for both contracts.
    // This admin is different from our custom defined admin and is solely used for migrations.
    .add_messages(set_wasm_admin_msgs);

    // Now configure the base asset of the account.
    // This contract is still the owner of the proxy at this point.
    if let Some(set_base_asset_msg) = set_base_asset_msg {
        resp = resp.add_message(set_base_asset_msg);
    }
    // Claim its namespace if applicable.
    if let Some(set_namespace_msg) = set_namespace_msg {
        resp = resp.add_message(set_namespace_msg);
    }

    resp = resp
        // And now transfer the admin rights to the manager.
        .add_message(set_proxy_admin_msg)
        // Set the proxy address on the manager.
        // This last step will trigger the installation of the modules.
        .add_message(add_proxy_address_msg);

    Ok(resp)
}

// Only owner can execute it
#[allow(clippy::too_many_arguments)]
pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    ans_host_contract: Option<String>,
    version_control_contract: Option<String>,
    module_factory_address: Option<String>,
) -> AccountFactoryResult {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut config: Config = CONFIG.load(deps.storage)?;

    if let Some(ans_host_contract) = ans_host_contract {
        // validate address format
        config.ans_host_contract = deps.api.addr_validate(&ans_host_contract)?;
    }

    if let Some(version_control_contract) = version_control_contract {
        // validate address format
        config.version_control_contract = deps.api.addr_validate(&version_control_contract)?;
    }

    if let Some(module_factory_address) = module_factory_address {
        // validate address format
        config.module_factory_address = deps.api.addr_validate(&module_factory_address)?;
    }
    CONFIG.save(deps.storage, &config)?;

    Ok(AccountFactoryResponse::action("update_config"))
}
