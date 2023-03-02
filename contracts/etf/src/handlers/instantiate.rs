use crate::contract::INSTANTIATE_REPLY_ID;
use crate::contract::{EtfApp, EtfResult, DEFAULT_LP_TOKEN_NAME, DEFAULT_LP_TOKEN_SYMBOL};
use abstract_os::etf::state::{State, FEE, STATE};
use abstract_os::etf::EtfInstantiateMsg;
use abstract_os::objects::fee::Fee;
use cosmwasm_std::{
    to_binary, Addr, DepsMut, Env, MessageInfo, ReplyOn, Response, SubMsg, WasmMsg,
};
use cw20::MinterResponse;
use cw20_base::msg::InstantiateMsg as TokenInstantiateMsg;

pub fn instantiate_handler(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    _etf: EtfApp,
    msg: EtfInstantiateMsg,
) -> EtfResult {
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
        id: INSTANTIATE_REPLY_ID,
        reply_on: ReplyOn::Success,
    }))
}
