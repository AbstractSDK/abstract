#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult};
use cw2::set_contract_version;
use wynd_lsd_hub::msg::{
    ConfigResponse as HubConfigResponse, QueryMsg as HubQueryMsg, SupplyResponse,
};
use wyndex::lp_converter::ExecuteMsg;

use crate::error::ContractError;
use crate::msg::{InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

const WITHDRAW_LIQUIDITY_REPLY_ID: u64 = 1;
const BOND_REPLY_ID: u64 = 2;
const PROVIDE_LIQUIDITY_REPLY_ID: u64 = 3;

// version info for migration info
pub const CONTRACT_NAME: &str = "crates.io:wynd-lp-converter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let hub_contract = deps.api.addr_validate(&msg.hub)?;

    // query hub contract for the liquidity token and bonded denom
    let hub_config: HubConfigResponse = deps
        .querier
        .query_wasm_smart(&hub_contract, &HubQueryMsg::Config {})?;
    let hub_supply: SupplyResponse = deps
        .querier
        .query_wasm_smart(&hub_contract, &HubQueryMsg::Supply {})?;

    // save this for later use in the config
    let config = Config {
        hub_contract,
        token_contract: hub_config.token_contract,
        base_denom: hub_supply.supply.bond_denom,
    };
    CONFIG.save(deps.storage, &config)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Convert {
            sender,
            amount,
            unbonding_period,
            pair_contract_from,
            pair_contract_to,
        } => execute::convert(
            deps,
            sender,
            amount,
            unbonding_period,
            pair_contract_from,
            pair_contract_to,
        ),
    }
}

/// The entry point to the contract for processing replies from submessages.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        WITHDRAW_LIQUIDITY_REPLY_ID => reply::withdraw_liquidity(deps, env),
        BOND_REPLY_ID => reply::bond(deps, env),
        PROVIDE_LIQUIDITY_REPLY_ID => reply::provide_liquidity(deps, env),
        _ => Err(ContractError::UnknownReplyId {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

mod execute {
    use cosmwasm_std::{to_binary, SubMsg, Uint128, WasmMsg};
    use cw20::Cw20ExecuteMsg;
    use wyndex::{
        asset::AssetInfoValidated,
        pair::{Cw20HookMsg, PairInfo, QueryMsg as PairQueryMsg},
    };

    use crate::state::{TmpData, TMP_DATA};

    use super::*;

    pub fn convert(
        deps: DepsMut,
        lp_owner: String,
        amount: Uint128,
        unbonding_period: u64,
        pair_contract_from: String,
        pair_contract_to: String,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        let lp_owner = deps.api.addr_validate(&lp_owner)?;
        let pair_contract_from = deps.api.addr_validate(&pair_contract_from)?;
        let pair_contract_to = deps.api.addr_validate(&pair_contract_to)?;

        let pair_info_from: PairInfo = deps
            .querier
            .query_wasm_smart(&pair_contract_from, &PairQueryMsg::Pair {})?;

        // go through the assets of `pair_contract_from` and replace the base denom with the token contract
        let assets: Vec<_> = pair_info_from
            .asset_infos
            .into_iter()
            .map(|asset| match asset {
                AssetInfoValidated::Native(denom) if denom == config.base_denom => {
                    AssetInfoValidated::Token(config.token_contract.clone())
                }
                _ => asset,
            })
            .collect();

        // save the data we need for the replies
        TMP_DATA.save(
            deps.storage,
            &TmpData {
                lp_owner,
                pair_contract_to,
                unbonding_period,
                assets,
            },
        )?;

        // withdraw liquidity from source pair
        // to do this, we need to send the LP tokens to the pair contract
        let resp = Response::new().add_submessage(SubMsg::reply_on_success(
            WasmMsg::Execute {
                contract_addr: pair_info_from.liquidity_token.into_string(),
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: pair_contract_from.into_string(),
                    amount,
                    msg: to_binary(&Cw20HookMsg::WithdrawLiquidity { assets: vec![] })?,
                })?,
                funds: vec![],
            },
            WITHDRAW_LIQUIDITY_REPLY_ID,
        ));

        Ok(resp)
    }
}

mod reply {
    use cosmwasm_std::{to_binary, Coin, Decimal, SubMsg, WasmMsg};
    use cw20::Cw20ExecuteMsg;
    use wynd_lsd_hub::msg::ExecuteMsg as HubExecuteMsg;
    use wyndex::stake::ReceiveMsg;
    use wyndex::{
        asset::{AssetInfo, AssetInfoExt},
        pair::{ExecuteMsg as PairExecuteMsg, PairInfo, QueryMsg as PairQueryMsg},
        querier::query_token_balance,
    };

    use crate::state::TMP_DATA;

    use super::*;

    /// Called after the liquidity has been withdrawn from the source pair contract.
    ///
    /// At this point, we should have the assets from the pair contract.
    /// One of them is the `base_denom` (e.g. juno) which needs to be sent to the hub contract.
    pub fn withdraw_liquidity(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
        let config: Config = CONFIG.load(deps.storage)?;

        // check how much base denom we got
        let amount = deps
            .querier
            .query_balance(env.contract.address, &config.base_denom)?
            .amount;

        // send the base denom to the hub contract
        let resp = Response::new().add_submessage(SubMsg::reply_on_success(
            WasmMsg::Execute {
                contract_addr: config.hub_contract.into_string(),
                msg: to_binary(&HubExecuteMsg::Bond {})?,
                funds: vec![Coin {
                    denom: config.base_denom,
                    amount,
                }],
            },
            BOND_REPLY_ID,
        ));

        Ok(resp)
    }

    /// Called after the base denom was bonded to the hub contract.
    ///
    /// At this point, we should have the wyAsset from the hub contract.
    /// We need to send this (together with the other assets) to the target pair contract.
    pub fn bond(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
        let tmp_data = TMP_DATA.load(deps.storage)?;

        // check how much of each asset we got
        let assets = tmp_data
            .assets
            .into_iter()
            .map(|asset| {
                asset
                    .query_balance(&deps.querier, &env.contract.address)
                    .map(|amt| AssetInfo::from(asset).with_balance(amt))
            })
            .collect::<StdResult<Vec<_>>>()?;

        // native assets need to be sent to the target pair contract as funds
        let funds: Vec<_> = assets
            .iter()
            .filter_map(|a| match &a.info {
                AssetInfo::Native(denom) => Some(Coin {
                    denom: denom.clone(),
                    amount: a.amount,
                }),
                _ => None,
            })
            .collect();
        // cw20 assets need to have their allowance increased
        let mut resp = Response::new();
        for asset in &assets {
            if let AssetInfo::Token(cw20) = &asset.info {
                resp = resp.add_message(WasmMsg::Execute {
                    contract_addr: cw20.clone(),
                    msg: to_binary(&Cw20ExecuteMsg::IncreaseAllowance {
                        spender: tmp_data.pair_contract_to.to_string(),
                        amount: asset.amount,
                        expires: None,
                    })?,
                    funds: vec![],
                })
            }
        }

        // send the wyAsset to the target pair contract
        let resp = resp.add_submessage(SubMsg::reply_on_success(
            WasmMsg::Execute {
                contract_addr: tmp_data.pair_contract_to.into_string(),
                msg: to_binary(&PairExecuteMsg::ProvideLiquidity {
                    assets,
                    slippage_tolerance: Some(Decimal::percent(50)), // this is the max allowed slippage
                    receiver: None, // we receive the LP tokens back, since we are the sender
                })?,
                funds,
            },
            PROVIDE_LIQUIDITY_REPLY_ID,
        ));

        Ok(resp)
    }

    /// Called after the liquidity was provided to the target pair.
    ///
    /// At this point, we should have the target pair LP tokens.
    /// We need to stake them to the target pair staking contract.
    pub fn provide_liquidity(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
        let tmp_data = TMP_DATA.load(deps.storage)?;

        // check how many LP tokens we got
        let pair_info_to: PairInfo = deps
            .querier
            .query_wasm_smart(tmp_data.pair_contract_to, &PairQueryMsg::Pair {})?;
        let lp_balance = query_token_balance(
            &deps.querier,
            &pair_info_to.liquidity_token,
            env.contract.address,
        )?;

        // send the LP tokens to the staking contract
        let resp = Response::new().add_message(WasmMsg::Execute {
            contract_addr: pair_info_to.liquidity_token.into_string(),
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: pair_info_to.staking_addr.into_string(),
                amount: lp_balance,
                msg: to_binary(&ReceiveMsg::Delegate {
                    unbonding_period: tmp_data.unbonding_period,
                    delegate_as: Some(tmp_data.lp_owner.into_string()), // this avoids another reply
                })?,
            })?,
            funds: vec![],
        });

        Ok(resp)
    }
}
