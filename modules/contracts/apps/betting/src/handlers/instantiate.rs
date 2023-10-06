use abstract_sdk::AbstractResponse;
use cosmwasm_std::{
    DepsMut, Env, MessageInfo, Response,
};

use crate::contract::{EtfApp, EtfResult};
use crate::msg::BetInstantiateMsg;
use crate::state::{Config, COTFIG_2, State, STATE};

pub const INSTANTIATE_REPLY_ID: u64 = 1u64;

pub fn instantiate_handler(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    app: EtfApp,
    msg: BetInstantiateMsg,
) -> EtfResult {
    let state: State = State::default();
    let config = Config::default();

    // let lp_token_name: String = msg
    //     .token_name
    //     .unwrap_or_else(|| String::from(DEFAULT_LP_TOKEN_NAME));
    //
    // let lp_token_symbol: String = msg
    //     .token_symbol
    //     .unwrap_or_else(|| String::from(DEFAULT_LP_TOKEN_SYMBOL));

    STATE.save(deps.storage, &state)?;
    COTFIG_2.save(deps.storage, &config)?;
    // RAKE.save(deps.storage, &Fee::new(msg.fee)?)?;
    Ok(app.tag_response(Response::new(), "instantiate"))

    // Ok(Response::new().add_submessage(SubMsg {
    //     // Create LP token
    //     msg: WasmMsg::Instantiate {
    //         admin: None,
    //         code_id: msg.token_code_id,
    //         msg: to_binary(&TokenInstantiateMsg {
    //             name: lp_token_name.clone(),
    //             symbol: lp_token_symbol,
    //             decimals: 6,
    //             initial_balances: vec![],
    //             mint: Some(MinterResponse {
    //                 minter: env.contract.address.to_string(),
    //                 cap: None,
    //             }),
    //             marketing: None,
    //         })?,
    //         funds: vec![],
    //         label: format!("Abstract ETF Shares: {}", lp_token_name),
    //     }
    //     .into(),
    //     gas_limit: None,
    //     id: INSTANTIATE_REPLY_ID,
    //     reply_on: ReplyOn::Success,
    // }))
}
