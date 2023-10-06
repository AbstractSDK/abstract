use abstract_core::objects::fee::Fee;
use abstract_sdk::AbstractResponse;
use abstract_sdk::features::AbstractNameService;
use cosmwasm_std::{Decimal, DepsMut, Env, MessageInfo, Response};

use crate::contract::{BetApp, BetResult};
use crate::msg::BetInstantiateMsg;
use crate::state::{Config, CONFIG, DEFAULT_RAKE_PERCENT, State, STATE};

pub const INSTANTIATE_REPLY_ID: u64 = 1u64;

pub fn instantiate_handler(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    app: BetApp,
    msg: BetInstantiateMsg,
) -> BetResult {
    let state: State = State::default();
    STATE.save(deps.storage, &state)?;

    let config = Config {
        rake: Fee::new(msg.rake.unwrap_or(Decimal::percent(DEFAULT_RAKE_PERCENT)))?,
        bet_asset: msg.bet_asset
    };

    config.validate(deps.as_ref(), &app)?;
    CONFIG.save(deps.storage, &config)?;


    // let lp_token_name: String = msg
    //     .token_name
    //     .unwrap_or_else(|| String::from(DEFAULT_LP_TOKEN_NAME));
    //
    // let lp_token_symbol: String = msg
    //     .token_symbol
    //     .unwrap_or_else(|| String::from(DEFAULT_LP_TOKEN_SYMBOL));

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
