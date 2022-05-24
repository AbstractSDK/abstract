use cosmwasm_std::{Coin, CosmosMsg, Deps, Env, Fraction, MessageInfo, Response, Uint128};

use abstract_os::core::proxy::msg::send_to_proxy;
use abstract_os::modules::dapp_base::common::{ANCHOR_MONEY_MARKET_ID, AUST_TOKEN_ID};
use abstract_os::queries::anchor::query_aust_exchange_rate;
use abstract_os::util::anchor::{anchor_deposit_msg, anchor_withdraw_msg};
use abstract_os::util::denom::UST_DENOM;

use crate::contract::{AnchorApi, AnchorResult};

// Add the custom dapp-specific message commands here

/// Constructs and forwards the anchor deposit_stable message for the proxy
/// The scenario covered here is such that there is UST in the proxy (or whatever similar framework you attach this dapp too)
/// and the anchor-dapp acts as an envoy preparing and providing the message to the proxy for execution
/// Caller address -> anchor-dapp -> Treasury executes message prepared by the anchor-dapp invoked by the caller address which is an admin
pub fn handle_deposit_stable(
    deps: Deps,
    _env: Env,
    msg_info: MessageInfo,
    dapp: AnchorApi,
    ust_deposit_amount: Uint128,
) -> AnchorResult {
    let state = dapp.base_state.load(deps.storage)?;
    // Check if caller is trader.
    state.assert_authorized_trader(&msg_info.sender)?;

    let proxy_address = &state.proxy_address;

    // Get anchor money market address
    let anchor_address = state
        .memory
        .query_contract(deps, &String::from(ANCHOR_MONEY_MARKET_ID))?;

    let mut messages: Vec<CosmosMsg> = vec![];
    // Prepare a deposit_msg using the provided info.
    // The anchor dapp will then use this message and pass it to the proxy for execution
    let deposit_msg: CosmosMsg = anchor_deposit_msg(
        deps,
        anchor_address,
        Coin::new(ust_deposit_amount.u128(), UST_DENOM),
    )?;
    println!("{:?}", deposit_msg);
    messages.push(deposit_msg);
    Ok(Response::new().add_message(send_to_proxy(messages, proxy_address)?))
}

/// Constructs and forwards the anchor redeem_stable message for the proxy
/// The scenario covered here is such that there is aUST in the proxy (or whatever similar framework you attach this dapp too)
/// and the anchor-dapp acts as an envoy preparing and providing the message to the proxy for execution
/// Caller address -> anchor-dapp -> Treasury executes message prepared by the anchor-dapp invoked by the caller address which is an admin
pub fn handle_redeem_stable(
    deps: Deps,
    _env: Env,
    info: MessageInfo,
    dapp: AnchorApi,
    ust_to_withdraw: Uint128,
) -> AnchorResult {
    let state = dapp.base_state.load(deps.storage)?;
    // Check if caller is trader.
    state.assert_authorized_trader(&info.sender)?;

    let proxy_address = &state.proxy_address;

    // Get anchor money market address
    let anchor_address = state
        .memory
        .query_contract(deps, &String::from(ANCHOR_MONEY_MARKET_ID))?;

    // Get aUST address
    let aust_address = state
        .memory
        .query_contract(deps, &String::from(AUST_TOKEN_ID))?;

    let mut messages: Vec<CosmosMsg> = vec![];

    let aust_exchange_rate = query_aust_exchange_rate(deps, anchor_address.to_string())?;

    // Prepare a deposit_msg using the provided info.
    // The anchor dapp will then use this message and pass it to the proxy for execution
    let withdraw_msg = anchor_withdraw_msg(
        aust_address,
        anchor_address,
        ust_to_withdraw * aust_exchange_rate.inv().unwrap(),
    )?;
    messages.push(withdraw_msg);
    Ok(Response::new().add_message(send_to_proxy(messages, proxy_address)?))
}
