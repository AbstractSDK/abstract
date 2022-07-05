#![allow(unused_imports)]
#![allow(unused_variables)]

use std::vec;

use abstract_add_on::{AddOnContract, AddOnResult};
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Reply, ReplyOn,
    Response, StdError, StdResult, SubMsg, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};
use cw_storage_plus::Map;
use protobuf::Message;
use semver::Version;

use abstract_os::objects::fee::Fee;
use abstract_os::VAULT;
use abstract_os::vault::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, StateResponse};
use cw20_base::msg::InstantiateMsg as TokenInstantiateMsg;

use crate::error::VaultError;
use crate::response::MsgInstantiateContractResponse;
use crate::state::{Pool, State, FEE, POOL, STATE};
use crate::{commands, queries};

const INSTANTIATE_REPLY_ID: u8 = 1u8;

const DEFAULT_LP_TOKEN_NAME: &str = "Vault LP token";
const DEFAULT_LP_TOKEN_SYMBOL: &str = "uvLP";

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type VaultDapp<'a> = AddOnContract<'a>;
pub type VaultResult = Result<Response, VaultError>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> VaultResult {
    let version: Version = CONTRACT_VERSION.parse()?;
    let storage_version: Version = get_contract_version(deps.storage)?.version.parse()?;
    if storage_version < version {
        set_contract_version(deps.storage, VAULT, CONTRACT_VERSION)?;
    }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(deps: DepsMut, env: Env, info: MessageInfo, msg: InstantiateMsg) -> VaultResult {
    let state: State = State {
        liquidity_token_addr: Addr::unchecked(""),
        provider_addr: deps.api.addr_validate(msg.provider_addr.as_str())?,
    };

    let lp_token_name: String = msg
        .vault_lp_token_name
        .unwrap_or_else(|| String::from(DEFAULT_LP_TOKEN_NAME));

    let lp_token_symbol: String = msg
        .vault_lp_token_symbol
        .unwrap_or_else(|| String::from(DEFAULT_LP_TOKEN_SYMBOL));

    STATE.save(deps.storage, &state)?;
    POOL.save(
        deps.storage,
        &Pool {
            deposit_asset: msg.deposit_asset.clone(),
            assets: vec![msg.deposit_asset],
        },
    )?;
    FEE.save(deps.storage, &Fee { share: msg.fee })?;

    VaultDapp::default().instantiate(deps, env.clone(), info, msg.base, VAULT, CONTRACT_VERSION)?;

    Ok(Response::new().add_submessage(SubMsg {
        // Create LP token
        msg: WasmMsg::Instantiate {
            admin: None,
            code_id: msg.token_code_id,
            msg: to_binary(&TokenInstantiateMsg {
                name: lp_token_name,
                symbol: lp_token_symbol,
                decimals: 6,
                initial_balances: vec![],
                mint: Some(MinterResponse {
                    minter: env.contract.address.to_string(),
                    cap: None,
                }),
                marketing: None,
            })?,
            funds: vec![],
            label: "White Whale Vault LP".to_string(),
        }
        .into(),
        gas_limit: None,
        id: u64::from(INSTANTIATE_REPLY_ID),
        reply_on: ReplyOn::Success,
    }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> VaultResult {
    let dapp = VaultDapp::default();
    match msg {
        ExecuteMsg::Base(dapp_msg) => dapp
            .execute(deps, env, info, dapp_msg)
            .map_err(VaultError::from),
        ExecuteMsg::Receive(msg) => commands::receive_cw20(deps, env, info, dapp, msg),
        ExecuteMsg::ProvideLiquidity { asset } => {
            // Check asset
            let asset = asset.check(deps.api, None)?;

            commands::try_provide_liquidity(deps, info, dapp, asset, None)
        }
        ExecuteMsg::UpdatePool {
            deposit_asset,
            assets_to_add,
            assets_to_remove,
        } => commands::update_pool(
            deps,
            info,
            dapp,
            deposit_asset,
            assets_to_add,
            assets_to_remove,
        ),
        ExecuteMsg::SetFee { fee } => commands::set_fee(deps, info, dapp, Fee { share: fee }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Base(dapp_msg) => VaultDapp::default().query(deps, env, dapp_msg),
        // handle dapp-specific queries here
        QueryMsg::State {} => to_binary(&StateResponse {
            liquidity_token: STATE.load(deps.storage)?.liquidity_token_addr.to_string(),
        }),
    }
}

/// This just stores the result for future query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    if msg.id == u64::from(INSTANTIATE_REPLY_ID) {
        let data = msg.result.unwrap().data.unwrap();
        let res: MsgInstantiateContractResponse = Message::parse_from_bytes(data.as_slice())
            .map_err(|_| {
                StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
            })?;
        let liquidity_token = res.get_contract_address();

        let api = deps.api;
        STATE.update(deps.storage, |mut meta| -> StdResult<_> {
            meta.liquidity_token_addr = api.addr_validate(liquidity_token)?;
            Ok(meta)
        })?;

        return Ok(Response::new().add_attribute("liquidity_token_addr", liquidity_token));
    }
    Ok(Response::default())
}
