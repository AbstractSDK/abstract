use std::convert::TryInto;

use cosmwasm_std::{
    from_binary, to_binary, Addr, CosmosMsg, Decimal, Deps, DepsMut, Env, Fraction, MessageInfo,
    Response, StdResult, Uint128, Uint64, WasmMsg,
};
use cw20::Cw20ReceiveMsg;
use cw_asset::{Asset, AssetInfo};
use pandora_os::core::proxy::msg::send_to_proxy;
use pandora_os::modules::dapp_base::state::{ADMIN, BASESTATE};
use pandora_os::native::version_control::msg::ExecuteMsg as VersionControlMsg;
use pandora_os::util::deposit_manager::Deposit;

use crate::contract::SubscriptionResult;
use crate::error::SubscriptionError;
use pandora_os::modules::add_ons::subscription::state::{IncomeAccumulator, State, CLIENTS, CONFIG, MONTH, STATE};
use pandora_os::modules::add_ons::subscription::msg::DepositHookMsg;

/// handler function invoked when the vault dapp contract receives
/// a transaction. In this case it is triggered when either a LP tokens received
/// by the contract or when the deposit asset is a cw20 asset.
pub fn receive_cw20(
    deps: DepsMut,
    _env: Env,
    msg_info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> SubscriptionResult {
    match from_binary(&cw20_msg.msg)? {
        DepositHookMsg::Pay { os_id } => {
            // Construct deposit asset
            let asset = Asset {
                info: AssetInfo::Cw20(msg_info.sender.clone()),
                amount: cw20_msg.amount,
            };
            try_pay(deps, msg_info, asset, Some(cw20_msg.sender), os_id)
        }
    }
}

/// Called when either paying with a native token or when paying
/// with a CW20.
pub fn try_pay(
    deps: DepsMut,
    msg_info: MessageInfo,
    asset: Asset,
    sender: Option<String>,
    os_id: u32,
) -> SubscriptionResult {
    // Load all needed states
    let config = CONFIG.load(deps.storage)?;
    let base_state = BASESTATE.load(deps.storage)?;
    // Get the liquidity provider address
    match sender {
        Some(addr) => Addr::unchecked(addr),
        None => {
            match asset.info {
                AssetInfo::Native(..) => {
                    // If native token, assert claimed amount is correct
                    let coin = msg_info.funds.last().unwrap().clone();
                    if Asset::native(coin.denom, coin.amount) != asset {
                        return Err(SubscriptionError::WrongNative {});
                    }
                    msg_info.sender
                }
                AssetInfo::Cw20(_) => return Err(SubscriptionError::NotUsingCW20Hook {}),
            }
        }
    };

    // Construct deposit info
    let deposit_info = config.payment_asset;

    // Assert payment asset and claimed asset infos are the same
    if deposit_info != asset.info {
        return Err(SubscriptionError::WrongToken {});
    }

    let mut customer_balance = CLIENTS.load(deps.storage, &os_id.to_be_bytes())?;
    customer_balance.increase((asset.amount.u128() as u64).into());

    CLIENTS
        .save(deps.storage, &os_id.to_be_bytes(), &customer_balance)?;

    // Init vector for logging
    let attrs = vec![
        ("Action:", String::from("Deposit to payment module")),
        ("Received funds:", asset.to_string()),
    ];

    Ok(Response::new().add_attributes(attrs).add_message(
        // Send the received asset to the proxy
        asset.transfer_msg(base_state.proxy_address)?,
    ))
}

/// Uses accumulator page mapping to process all active clients
fn tally_income(mut deps: DepsMut, env: Env, page_limit: Option<u32>) -> StdResult<()> {
    if let Some(res) = CLIENTS.page_with_accumulator(deps.branch(), page_limit, process_client)? {
        STATE.save(
            deps.storage,
            &State {
                income: Uint64::from(res.income),
                next_pay_day: (env.block.time.seconds() + MONTH).into(),
                debtors: res.debtors,
            },
        )?;
    }
    Ok(())
}

fn process_client(variables: (Vec<u8>, Deposit, Deps), acc: &mut IncomeAccumulator) {
    let (key, mut deposit, deps) = variables;
    let os_id = u32::from_be_bytes(key.try_into().unwrap());
    let subscription_cost = CONFIG.load(deps.storage).unwrap().subscription_cost;

    match deposit.decrease(subscription_cost).ok() {
        Some(_) => {
            acc.income += subscription_cost.u64() as u32;
        }
        None => {
            acc.debtors.push(os_id);
        }
    }
}

pub fn purge_debtors(mut deps: DepsMut, env: Env, page_limit: Option<u32>) -> SubscriptionResult {
    let mut state: State = STATE.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    // First tally total income
    if state.next_pay_day.u64() < env.block.time.seconds() {
        tally_income(deps.branch(), env, page_limit)?;
        let info = CLIENTS.status.load(deps.storage)?;
        return Ok(Response::new().add_attributes(vec![
            ("Action:", String::from("Tally income")),
            ("Progress:", info.progress()),
        ]));
    };

    let final_length = state
        .debtors
        .len()
        .saturating_sub(page_limit.unwrap_or_else(|| 10u32) as usize);
    let remove_from_active_set: Vec<u32> = state.debtors.drain(final_length..).collect();

    // TODO: Remove contributors from this list

    Ok(
        Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.version_control_address.into(),
            msg: to_binary(&VersionControlMsg::RemoveDebtors {
                os_ids: remove_from_active_set,
            })?,
            funds: vec![],
        })),
    )
}
