use abstract_core::objects::{AssetEntry, PoolAddress};
use abstract_moneymarket_standard::{
    ans_action::{pool_address, WholeMoneymarketAction},
    msg::{
        GenerateMessagesResponse, MoneymarketExecuteMsg, MoneymarketFeesResponse,
        MoneymarketQueryMsg,
    },
    MoneymarketError,
};
use abstract_sdk::features::AbstractNameService;
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdError};

use crate::{
    contract::{MoneymarketAdapter, MoneymarketResult},
    exchanges::exchange_resolver::{self, resolve_exchange},
    handlers::query::exchange_resolver::is_over_ibc,
    state::MONEYMARKET_FEES,
};
use cw_asset::{Asset, AssetInfo, AssetInfoBase};

pub fn query_handler(
    deps: Deps,
    env: Env,
    adapter: &MoneymarketAdapter,
    msg: MoneymarketQueryMsg,
) -> MoneymarketResult<Binary> {
    match msg {
        MoneymarketQueryMsg::SimulateSwapRaw {
            offer_asset,
            ask_asset,
            moneymarket,
            pool,
        } => {
            let simulate_response = simulate_swap(
                deps,
                env,
                moneymarket,
                pool.check(deps.api)?,
                offer_asset.check(deps.api, None)?,
                ask_asset.check(deps.api, None)?,
            )?;

            to_json_binary(&simulate_response).map_err(Into::into)
        }
        MoneymarketQueryMsg::GenerateMessages {
            mut message,
            addr_as_sender,
        } => {
            if let MoneymarketExecuteMsg::AnsAction {
                moneymarket,
                action,
            } = message
            {
                let ans = adapter.name_service(deps);
                let whole_moneymarket_action = WholeMoneymarketAction(moneymarket.clone(), action);
                message = MoneymarketExecuteMsg::RawAction {
                    moneymarket,
                    action: ans.query(&whole_moneymarket_action)?,
                }
            }
            match message {
                MoneymarketExecuteMsg::RawAction {
                    moneymarket,
                    action,
                } => {
                    let (local_moneymarket_name, is_over_ibc) = is_over_ibc(env, &moneymarket)?;
                    // if exchange is on an app-chain, execute the action on the app-chain
                    if is_over_ibc {
                        return Err(MoneymarketError::IbcMsgQuery);
                    }
                    let exchange = exchange_resolver::resolve_exchange(&local_moneymarket_name)?;
                    let addr_as_sender = deps.api.addr_validate(&addr_as_sender)?;
                    let (messages, _) =
                        crate::adapter::MoneymarketAdapter::resolve_moneymarket_action(
                            adapter,
                            deps,
                            addr_as_sender,
                            action,
                            exchange,
                        )?;
                    to_json_binary(&GenerateMessagesResponse { messages }).map_err(Into::into)
                }
                _ => Err(MoneymarketError::InvalidGenerateMessage {}),
            }
        }
        MoneymarketQueryMsg::Fees {} => fees(deps),
    }
}

pub fn fees(deps: Deps) -> MoneymarketResult<Binary> {
    let moneymarket_fees = MONEYMARKET_FEES.load(deps.storage)?;
    let resp = MoneymarketFeesResponse {
        swap_fee: moneymarket_fees.swap_fee(),
        recipient: moneymarket_fees.recipient,
    };
    to_json_binary(&resp).map_err(Into::into)
}

pub fn simulate_swap(
    deps: Deps,
    _env: Env,
    moneymarket: String,
    pool: PoolAddress,
    mut offer_asset: Asset,
    ask_asset: AssetInfo,
) -> MoneymarketResult<SimulateSwapResponse<AssetInfoBase<String>>> {
    let exchange =
        resolve_exchange(&moneymarket).map_err(|e| StdError::generic_err(e.to_string()))?;

    let pool_info = MoneymarketAssetPairing::new(
        offer_asset.info.clone().into(),
        ask_asset.clone().into(),
        exchange.name(),
    );

    // compute adapter fee
    let moneymarket_fees = MONEYMARKET_FEES.load(deps.storage)?;
    let adapter_fee = moneymarket_fees.swap_fee().compute(offer_asset.amount);
    offer_asset.amount -= adapter_fee;

    let (return_amount, spread_amount, commission_amount, fee_on_input) = exchange
        .simulate_swap(deps, pool, offer_asset.clone(), ask_asset.clone())
        .map_err(|e| StdError::generic_err(e.to_string()))?;
    let commission_asset = if fee_on_input {
        ask_asset
    } else {
        offer_asset.info
    };

    let resp = SimulateSwapResponse {
        pool: pool_info,
        return_amount,
        spread_amount,
        commission: (commission_asset.into(), commission_amount),
        usage_fee: adapter_fee,
    };
    Ok(resp)
}
