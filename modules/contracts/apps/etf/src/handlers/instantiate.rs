use abstract_app::std::objects::fee::Fee;
use cosmwasm_std::{
    to_json_binary, Addr, DepsMut, Env, MessageInfo, ReplyOn, Response, SubMsg, WasmMsg,
};
use cw20::MinterResponse;
use cw20_base::msg::InstantiateMsg as TokenInstantiateMsg;

use crate::{
    contract::{EtfApp, EtfResult, DEFAULT_LP_TOKEN_NAME, DEFAULT_LP_TOKEN_SYMBOL},
    msg::EtfInstantiateMsg,
    state::{State, FEE, STATE},
};

pub const INSTANTIATE_REPLY_ID: u64 = 1u64;

pub fn instantiate_handler(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    _etf: EtfApp,
    msg: EtfInstantiateMsg,
) -> EtfResult {
    let state: State = State {
        share_token_address: Addr::unchecked(""),
        manager_addr: deps.api.addr_validate(msg.manager_addr.as_str())?,
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
            msg: to_json_binary(&TokenInstantiateMsg {
                name: lp_token_name.clone(),
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
            label: format!("Abstract ETF Shares: {}", lp_token_name),
        }
        .into(),
        gas_limit: None,
        id: INSTANTIATE_REPLY_ID,
        reply_on: ReplyOn::Success,
    }))
}
