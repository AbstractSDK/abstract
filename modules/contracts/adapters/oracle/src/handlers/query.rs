use crate::contract::{OracleAdapter, OracleResult};
use abstract_core::objects::{AssetEntry, DexAssetPairing, PoolAddress};
use abstract_oracle_standard::msg::OracleQueryMsg;
use abstract_sdk::features::AbstractNameService;
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdError};
use cw_asset::{Asset, AssetInfo, AssetInfoBase};

pub fn query_handler(
    deps: Deps,
    env: Env,
    adapter: &OracleAdapter,
    msg: OracleQueryMsg,
) -> OracleResult<Binary> {
    match msg {
        DexQueryMsg::SimulateSwapRaw {
            offer_asset,
            ask_asset,
            dex,
            pool,
        } => {
            let simulate_response = simulate_swap(
                deps,
                env,
                dex,
                pool.check(deps.api)?,
                offer_asset.check(deps.api, None)?,
                ask_asset.check(deps.api, None)?,
            )?;

            to_json_binary(&simulate_response).map_err(Into::into)
        }
        DexQueryMsg::GenerateMessages {
            mut message,
            addr_as_sender,
        } => {
            if let DexExecuteMsg::AnsAction { dex, action } = message {
                let ans = adapter.name_service(deps);
                let whole_dex_action = WholeDexAction(dex.clone(), action);
                message = DexExecuteMsg::RawAction {
                    dex,
                    action: ans.query(&whole_dex_action)?,
                }
            }
            match message {
                DexExecuteMsg::RawAction { dex, action } => {
                    let (local_dex_name, is_over_ibc) = is_over_ibc(env, &dex)?;
                    // if exchange is on an app-chain, execute the action on the app-chain
                    if is_over_ibc {
                        return Err(DexError::IbcMsgQuery);
                    }
                    let exchange = oracle_resolver::resolve_exchange(&local_dex_name)?;
                    let addr_as_sender = deps.api.addr_validate(&addr_as_sender)?;
                    let (messages, _) = crate::adapter::DexAdapter::resolve_dex_action(
                        adapter,
                        deps,
                        addr_as_sender,
                        action,
                        exchange,
                    )?;
                    to_json_binary(&GenerateMessagesResponse { messages }).map_err(Into::into)
                }
                _ => Err(DexError::InvalidGenerateMessage {}),
            }
        }
        DexQueryMsg::Fees {} => fees(deps),
        DexQueryMsg::SimulateSwap {
            offer_asset,
            ask_asset,
            dex,
        } => {
            let ans = adapter.name_service(deps);
            let cw_offer_asset = ans.query(&offer_asset)?;
            let cw_ask_asset = ans.query(&ask_asset)?;

            let pool_address = pool_address(
                dex.clone(),
                (offer_asset.name.clone(), ask_asset.clone()),
                &deps.querier,
                ans.host(),
            )?;

            let simulate_response = simulate_swap(
                deps,
                env,
                dex.clone(),
                pool_address,
                cw_offer_asset,
                cw_ask_asset.clone(),
            )?;

            // We return ans assets here
            let resp = SimulateSwapResponse::<AssetEntry> {
                pool: DexAssetPairing::new(offer_asset.name.clone(), ask_asset.clone(), &dex),
                return_amount: simulate_response.return_amount,
                spread_amount: simulate_response.spread_amount,
                commission: if simulate_response.commission.0 == cw_ask_asset.into() {
                    (ask_asset, simulate_response.commission.1)
                } else {
                    (offer_asset.name, simulate_response.commission.1)
                },
                usage_fee: simulate_response.usage_fee,
            };
            to_json_binary(&resp).map_err(Into::into)
        }
    }
}
