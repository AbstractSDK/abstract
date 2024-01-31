use abstract_core::{
    manager::ModuleInstallConfig,
    module_factory::SimulateInstallModulesResponse,
    objects::{
        account::{generate_account_salt, AccountTrace},
        module::assert_module_data_validity,
        AccountId, AssetEntry, ABSTRACT_ACCOUNT_ID,
    },
    AbstractError,
};
use abstract_sdk::{
    core::{
        manager::InstantiateMsg as ManagerInstantiateMsg,
        objects::{
            gov_type::GovernanceDetails,
            module::{Module, ModuleInfo},
            module_reference::ModuleReference,
        },
        proxy::InstantiateMsg as ProxyInstantiateMsg,
        version_control::{
            AccountBase, ExecuteMsg as VCExecuteMsg, ModulesResponse, QueryMsg as VCQuery,
        },
        AbstractResult, MANAGER, PROXY,
    },
    feature_objects::VersionControlContract,
};
use cosmwasm_std::{
    ensure_eq, instantiate2_address, to_json_binary, Addr, Coins, CosmosMsg, DepsMut, Empty, Env,
    MessageInfo, QuerierWrapper, SubMsg, SubMsgResult, WasmMsg,
};

use crate::{
    contract::{AccountFactoryResponse, AccountFactoryResult},
    error::AccountFactoryError,
    state::*,
};

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
    account_id: Option<AccountId>,
) -> AccountFactoryResult {
    let config = CONFIG.load(deps.storage)?;
    let abstract_registry = VersionControlContract::new(config.version_control_contract.clone());

    let governance = governance.verify(deps.as_ref(), config.version_control_contract.clone())?;
    // If an account_id is provided, assert the caller is the ibc host and return the account_id.
    // Else get the next account id and set the origin to local.
    let account_id = if let Some(account_id) = account_id {
        // if the account_id is provided, assert that the caller is the ibc host
        let ibc_host = config
            .ibc_host
            .ok_or(AccountFactoryError::IbcHostNotSet {})?;
        ensure_eq!(
            info.sender,
            ibc_host,
            AccountFactoryError::SenderNotIbcHost(info.sender.into(), ibc_host.into())
        );
        // then assert that the account trace is remote and properly formatted
        account_id.trace().verify_remote()?;
        account_id
    } else {
        // else the call is local so we need to look up the account sequence
        // and set the origin to local
        let origin = AccountTrace::Local;

        // load the next account id
        // if it doesn't exist then it's the first account so set it to 0.
        let next_sequence = LOCAL_ACCOUNT_SEQUENCE.may_load(deps.storage)?.unwrap_or(0);

        // Check if the caller is the owner when instantiating the abstract account
        if next_sequence == ABSTRACT_ACCOUNT_ID.seq() {
            cw_ownable::assert_owner(deps.storage, &info.sender)?;
        }
        AccountId::new(next_sequence, origin)?
    };

    // Query version_control for code_id of Proxy and Module contract
    let proxy_module: Module =
        query_module(&deps.querier, &config.version_control_contract, PROXY)?;
    let manager_module: Module =
        query_module(&deps.querier, &config.version_control_contract, MANAGER)?;

    let simulate_resp: SimulateInstallModulesResponse = deps.querier.query_wasm_smart(
        config.module_factory_address.to_string(),
        &abstract_core::module_factory::QueryMsg::SimulateInstallModules {
            modules: install_modules.iter().map(|m| m.module.clone()).collect(),
        },
    )?;
    let funds_for_install = simulate_resp.total_required_funds;
    let funds_for_namespace_fee = if namespace.is_some() {
        abstract_registry
            .namespace_registration_fee(&deps.querier)?
            .into_iter()
            .collect()
    } else {
        vec![]
    };

    // Remove all funds used to install the module and account fee to pass rest to the proxy contract
    let mut funds_to_proxy = Coins::try_from(info.funds.clone()).unwrap();
    for coin in funds_for_install
        .clone()
        .into_iter()
        .chain(funds_for_namespace_fee.clone().into_iter())
    {
        funds_to_proxy.sub(coin).map_err(|_| {
            AbstractError::Fee(format!(
                "Invalid fee payment sent. Expected {:?}, sent {:?}",
                funds_for_install, info.funds
            ))
        })?;
    }

    let salt = generate_account_salt(&account_id);

    // Get code_ids
    let (proxy_code_id, manager_code_id) = if let (
        ModuleReference::AccountBase(proxy_code_id),
        ModuleReference::AccountBase(manager_code_id),
    ) = (
        proxy_module.reference.clone(),
        manager_module.reference.clone(),
    ) {
        (proxy_code_id, manager_code_id)
    } else {
        return Err(AccountFactoryError::WrongModuleKind(
            proxy_module.info.to_string(),
            "account_base".to_string(),
        ));
    };

    // Get checksums
    let proxy_checksum = deps.querier.query_wasm_code_info(proxy_code_id)?.checksum;
    let manager_checksum = deps.querier.query_wasm_code_info(manager_code_id)?.checksum;

    let proxy_addr = instantiate2_address(
        &proxy_checksum,
        &deps.api.addr_canonicalize(env.contract.address.as_str())?,
        salt.as_slice(),
    )?;
    let proxy_addr_human = deps.api.addr_humanize(&proxy_addr)?;
    let manager_addr = instantiate2_address(
        &manager_checksum,
        &deps.api.addr_canonicalize(env.contract.address.as_str())?,
        salt.as_slice(),
    )?;
    let manager_addr_human = deps.api.addr_humanize(&manager_addr)?;

    let account_base = AccountBase {
        manager: manager_addr_human,
        proxy: proxy_addr_human,
    };
    // save context for after-init check
    let context = Context {
        account_id,
        account_base: account_base.clone(),
        manager_module,
        proxy_module,
    };
    CONTEXT.save(deps.storage, &context)?;

    let proxy_message = ProxyInstantiateMsg {
        account_id: context.account_id,
        ans_host_address: config.ans_host_contract.to_string(),
        manager_addr: context.account_base.manager.to_string(),
        base_asset,
    };

    // Add Account base to version_control
    let add_account_to_version_control_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.version_control_contract.to_string(),
        funds: funds_for_namespace_fee,
        msg: to_json_binary(&VCExecuteMsg::AddAccount {
            account_id: proxy_message.account_id.clone(),
            account_base: context.account_base,
            namespace,
        })?,
    });

    // The execution order here is important.
    // Installing modules on the manager account requires that:
    // - The account is registered.
    // - The proxy is instantiated.
    // - The manager instantiated and proxy is registered on the manager.
    // (this last step triggers the installation of the modules.)
    Ok(AccountFactoryResponse::new(
        "create_account",
        vec![
            (
                "account_sequence",
                &proxy_message.account_id.seq().to_string(),
            ),
            ("trace", &proxy_message.account_id.trace().to_string()),
        ],
    )
    // So first register account on version control
    .add_message(add_account_to_version_control_msg)
    // Then instantiate proxy
    .add_message(WasmMsg::Instantiate2 {
        code_id: proxy_code_id,
        funds: funds_to_proxy.into_vec(),
        admin: Some(account_base.manager.to_string()),
        label: format!("Proxy of Account: {}", proxy_message.account_id),
        msg: to_json_binary(&proxy_message)?,
        salt: salt.clone(),
    })
    // Instantiate manager and install apps
    // And validate contract versions in a callback
    .add_submessage(SubMsg::reply_on_success(
        WasmMsg::Instantiate2 {
            code_id: manager_code_id,
            funds: funds_for_install,
            admin: Some(account_base.manager.into_string()),
            label: format!("Manager of Account: {}", proxy_message.account_id),
            msg: to_json_binary(&ManagerInstantiateMsg {
                account_id: proxy_message.account_id,
                owner: governance.into(),
                version_control_address: config.version_control_contract.into_string(),
                module_factory_address: config.module_factory_address.into_string(),
                proxy_addr: account_base.proxy.into_string(),
                name,
                description,
                link,
                install_modules,
            })?,
            salt,
        },
        CREATE_ACCOUNT_MANAGER_MSG_ID,
    )))
}

fn query_module(
    querier: &QuerierWrapper,
    version_control_addr: &Addr,
    module_id: &str,
) -> AbstractResult<Module> {
    let ModulesResponse { mut modules } = querier.query_wasm_smart(
        version_control_addr.to_string(),
        &VCQuery::Modules {
            infos: vec![ModuleInfo::from_id_latest(module_id)?],
        },
    )?;

    Ok(modules.swap_remove(0).module)
}

/// Validates instantiated manager and proxy modules
pub fn validate_instantiated_account(deps: DepsMut, _result: SubMsgResult) -> AccountFactoryResult {
    let context = CONTEXT.load(deps.storage)?;
    CONTEXT.remove(deps.storage);

    let account_base = context.account_base;
    let account_id = context.account_id;

    // assert proxy and manager contract information is correct
    assert_module_data_validity(
        &deps.querier,
        &context.manager_module,
        Some(account_base.manager.clone()),
    )?;
    assert_module_data_validity(
        &deps.querier,
        &context.proxy_module,
        Some(account_base.proxy.clone()),
    )?;

    // Add 1 to account sequence for local origin
    if account_id.is_local() {
        LOCAL_ACCOUNT_SEQUENCE.save(deps.storage, &account_id.seq().checked_add(1).unwrap())?;
    }

    let resp = AccountFactoryResponse::new(
        "create_account",
        vec![
            ("account", account_id.to_string()),
            ("manager_address", account_base.manager.into_string()),
            ("proxy_address", account_base.proxy.into_string()),
        ],
    );

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
    ibc_host: Option<String>,
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

    if let Some(ibc_host) = ibc_host {
        // validate address format
        config.ibc_host = Some(deps.api.addr_validate(&ibc_host)?);
    }
    CONFIG.save(deps.storage, &config)?;

    Ok(AccountFactoryResponse::action("update_config"))
}
