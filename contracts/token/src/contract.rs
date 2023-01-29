use crate::state::{Config, ADMIN, CONFIG};
use abstract_sdk::feature_objects::VersionControlContract;
use abstract_sdk::{
    os::abstract_token::{ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    *,
};
use cosmwasm_std::{
    to_binary, Addr, Api, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError,
    StdResult,
};
use cw2::set_contract_version;
use cw20::Cw20ExecuteMsg;
use cw20_base::{
    contract::{create_accounts, execute as cw20_execute, query as cw20_query},
    state::{MinterData, TokenInfo, TOKEN_INFO},
    ContractError,
};
use std::convert::TryInto;

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "pandora:token";
/// Contract version that is used for migration.
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// ## Description
/// Creates a new contract with the specified parameters in the [`InstantiateMsg`].
/// Returns the default object of type [`Response`] if the operation was successful,
/// or a [`ContractError`] if the contract was not created.
/// ## Params
/// * **deps** is the object of type [`DepsMut`].
///
/// * **_env** is the object of type [`Env`].
///
/// * **_info** is the object of type [`MessageInfo`].
/// * **msg** is a message of type [`InstantiateMsg`] which contains the basic settings for creating a contract.
#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Check valid token info
    msg.validate()?;

    // Create initial accounts
    let total_supply = create_accounts(&mut deps, msg.initial_balances.as_slice())?;

    // Check supply cap
    if let Some(limit) = msg.get_cap() {
        if total_supply > limit {
            return Err(StdError::generic_err("Initial supply greater than cap").into());
        }
    }

    let mint = match msg.mint {
        Some(m) => Some(MinterData {
            minter: addr_validate_to_lower(deps.api, &m.minter)?,
            cap: m.cap,
        }),
        None => None,
    };

    // Store token info
    let data = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply,
        mint,
    };

    TOKEN_INFO.save(deps.storage, &data)?;

    // Custom logic
    let config = Config {
        transfers_restricted: true,
        version_control_address: deps.api.addr_validate(&msg.version_control_address)?,
        whitelisted_addr: vec![],
    };

    CONFIG.save(deps.storage, &config)?;

    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateWhitelist {
            to_add,
            to_remove,
            restrict_transfers,
        } => update_whitelist(deps, info, to_add, to_remove, restrict_transfers),
        ExecuteMsg::UpdateAdmin { new_admin } => set_admin(deps, info, new_admin),
        e => {
            let cw20_msg = e.try_into()?;
            match &cw20_msg {
                Cw20ExecuteMsg::Transfer { recipient, .. }
                | Cw20ExecuteMsg::Send {
                    contract: recipient,
                    ..
                }
                | Cw20ExecuteMsg::TransferFrom { recipient, .. }
                | Cw20ExecuteMsg::SendFrom {
                    contract: recipient,
                    ..
                } => {
                    assert_recipient_allowed(deps.as_ref(), recipient)?;
                }
                _ => (),
            }
            cw20_execute(deps, env, info, cw20_msg)
        }
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => {
            let config = CONFIG.load(deps.storage)?;
            let admin = ADMIN.get(deps)?.unwrap();
            let whitelist = config
                .whitelisted_addr
                .iter()
                .map(ToString::to_string)
                .collect();
            to_binary(&ConfigResponse {
                transfers_restricted: config.transfers_restricted,
                version_control_address: config.version_control_address.to_string(),
                whitelisted_addr: whitelist,
                admin: admin.to_string(),
            })
        }
        e => cw20_query(deps, env, e.try_into()?),
    }
}

/// ## Description
/// Used for migration of contract. Returns the default object of type [`Response`].
/// ## Params
/// * **_deps** is the object of type [`DepsMut`].
///
/// * **_env** is the object of type [`Env`].
///
/// * **_msg** is the object of type [`MigrateMsg`].
#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}

fn addr_validate_to_lower(api: &dyn Api, addr: &str) -> StdResult<Addr> {
    if addr.to_lowercase() != addr {
        return Err(StdError::generic_err(format!(
            "Address {addr} should be lowercase"
        )));
    }
    api.addr_validate(addr)
}

fn assert_recipient_allowed(deps: Deps, recipient: &str) -> Result<(), ContractError> {
    // is recipient a whitelisted?
    let config = CONFIG.load(deps.storage)?;
    if config
        .whitelisted_addr
        .contains(&deps.api.addr_validate(recipient)?)
        || !config.transfers_restricted
    {
        return Ok(());
    }
    let verify_feature = VersionControlContract {
        address: config.version_control_address,
    };
    verify_feature
        .os_register(deps)
        .assert_proxy(&deps.api.addr_validate(recipient)?)
        .map_err(|_| StdError::generic_err("receiver must be a valid Abstract proxy contract"))?;
    Ok(())
}
fn set_admin(deps: DepsMut, info: MessageInfo, admin: String) -> Result<Response, ContractError> {
    let admin_addr = deps.api.addr_validate(&admin)?;
    // Admin is asserted here
    ADMIN
        .execute_update_admin::<Empty, Empty>(deps, info, Some(admin_addr))
        .map_err(|e| ContractError::Std(StdError::generic_err(e.to_string())))
}

fn update_whitelist(
    deps: DepsMut,
    msg_info: MessageInfo,
    to_add: Vec<String>,
    to_remove: Vec<String>,
    restrict_transfers: Option<bool>,
) -> Result<Response, ContractError> {
    // Only Admin can call this method
    ADMIN
        .assert_admin(deps.as_ref(), &msg_info.sender)
        .map_err(|_| ContractError::Unauthorized {})?;
    let mut config = CONFIG.load(deps.storage)?;
    for new_address in to_add.into_iter() {
        // validate addr
        let addr = deps.as_ref().api.addr_validate(&new_address)?;
        // Update function for new or existing keys
        if config.whitelisted_addr.contains(&addr) {
            return Err(ContractError::Std(StdError::generic_err(
                "address already whitelisted",
            )));
        }
        config.whitelisted_addr.push(addr);
    }

    config
        .whitelisted_addr
        .retain(|e| !to_remove.contains(&e.to_string()));
    if let Some(restict) = restrict_transfers {
        config.transfers_restricted = restict;
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "updated contract addresses"))
}
