use std::vec;

use abstract_add_on::AddOnContract;

use abstract_os::add_on::{ExecuteMsg, InstantiateMsg, QueryMsg};
use abstract_sdk::AbstractExecute;
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn,
    Response, StdError, StdResult, SubMsg, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::{Cw20ReceiveMsg, MinterResponse};

use protobuf::Message;
use semver::Version;

use abstract_os::etf::{EtfExecuteMsg, EtfInstantiateMsg, EtfQueryMsg, MigrateMsg, StateResponse};
use abstract_os::objects::fee::Fee;
use abstract_os::ETF;
use cw20_base::msg::InstantiateMsg as TokenInstantiateMsg;

use crate::commands::{self, receive_cw20};
use crate::error::VaultError;
use crate::response::MsgInstantiateContractResponse;
use crate::state::{State, FEE, STATE};

const INSTANTIATE_REPLY_ID: u8 = 1u8;

const DEFAULT_LP_TOKEN_NAME: &str = "ETF LP token";
const DEFAULT_LP_TOKEN_SYMBOL: &str = "etfLP";

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type EtfAddOn<'a> = AddOnContract<'a, EtfExecuteMsg, VaultError, Cw20ReceiveMsg>;
pub type EtfResult = Result<Response, VaultError>;

const ETF_ADDON: EtfAddOn<'static> = EtfAddOn::new().with_receive(receive_cw20);

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> EtfResult {
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    let storage_version: Version = get_contract_version(deps.storage)?.version.parse().unwrap();
    if storage_version < version {
        set_contract_version(deps.storage, ETF, CONTRACT_VERSION)?;
    }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg<EtfInstantiateMsg>,
) -> EtfResult {
    EtfAddOn::instantiate(
        deps.branch(),
        env.clone(),
        info,
        msg.base,
        ETF,
        CONTRACT_VERSION,
    )?;
    let msg = msg.custom;
    let state: State = State {
        liquidity_token_addr: Addr::unchecked(""),
        provider_addr: deps.api.addr_validate(msg.provider_addr.as_str())?,
    };

    let lp_token_name: String = msg
        .token_name
        .unwrap_or_else(|| String::from(DEFAULT_LP_TOKEN_NAME));

    let lp_token_symbol: String = msg
        .token_symbol
        .unwrap_or_else(|| String::from(DEFAULT_LP_TOKEN_SYMBOL));

    STATE.save(deps.storage, &state)?;
    FEE.save(deps.storage, &Fee::new(msg.fee)?)?;

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
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<EtfExecuteMsg, Cw20ReceiveMsg>,
) -> EtfResult {
    ETF_ADDON.execute(deps, env, info, msg, request_handler)
}

fn request_handler(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    vault: EtfAddOn,
    msg: EtfExecuteMsg,
) -> EtfResult {
    match msg {
        EtfExecuteMsg::ProvideLiquidity { asset } => {
            // Check asset
            let asset = asset.check(deps.api, None)?;
            commands::try_provide_liquidity(deps, info, vault, asset, None)
        }
        EtfExecuteMsg::SetFee { fee } => commands::set_fee(deps, info, vault, fee),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg<EtfQueryMsg>) -> StdResult<Binary> {
    match msg {
        QueryMsg::Base(dapp_msg) => ETF_ADDON.query(deps, env, dapp_msg),
        // handle dapp-specific queries here
        QueryMsg::AddOn(EtfQueryMsg::State {}) => {
            let fee = FEE.load(deps.storage)?;
            to_binary(&StateResponse {
                liquidity_token: STATE.load(deps.storage)?.liquidity_token_addr.to_string(),
                fee: fee.share(),
            })
        }
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
