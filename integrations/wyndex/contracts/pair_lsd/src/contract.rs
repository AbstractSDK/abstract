use std::collections::HashMap;
use std::vec;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, ensure, from_binary, to_binary, wasm_execute, Addr, Binary, CosmosMsg, Decimal,
    Decimal256, Deps, DepsMut, Empty, Env, Fraction, MessageInfo, QuerierWrapper, Reply, Response,
    StdError, StdResult, Uint128, Uint256, WasmMsg,
};
use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use itertools::Itertools;

use wyndex::asset::{
    addr_opt_validate, check_swap_parameters, Asset, AssetInfo, AssetInfoValidated, AssetValidated,
    Decimal256Ext, DecimalAsset, MINIMUM_LIQUIDITY_AMOUNT,
};
use wyndex::common::{claim_ownership, drop_ownership_proposal, propose_new_owner};
use wyndex::factory::PairType;
use wyndex::fee_config::FeeConfig;
use wyndex::pair::{
    add_referral, assert_max_spread, check_asset_infos, check_assets, check_cw20_in_pool,
    create_lp_token, get_share_in_assets, handle_referral, handle_reply, migration_check,
    mint_token_message, save_tmp_staking_config, take_referral, ConfigResponse, Cw20HookMsg,
    InstantiateMsg, MigrateMsg, SpotPricePredictionResponse, SpotPriceResponse, StablePoolParams,
    StablePoolUpdateParams,
};
use wyndex::pair::{
    CumulativePricesResponse, ExecuteMsg, PairInfo, PoolResponse, QueryMsg,
    ReverseSimulationResponse, SimulationResponse, StablePoolConfig,
};
use wyndex::querier::{query_factory_config, query_fee_info, query_supply};
use wyndex::DecimalCheckedOps;

use crate::math::{
    calc_y, compute_d, AMP_PRECISION, MAX_AMP, MAX_AMP_CHANGE, MIN_AMP_CHANGING_TIME,
};
use crate::msg::{TargetQuery, TargetValueResponse};
use crate::state::{
    get_precision, store_precisions, Config, LsdData, CIRCUIT_BREAKER, CONFIG, FROZEN,
    OWNERSHIP_PROPOSAL,
};
use crate::utils::{
    accumulate_prices, adjust_precision, calc_new_price_a_per_b, calc_spot_price,
    compute_current_amp, compute_swap, find_spot_price, select_pools, SwapResult,
};
use wyndex::pair::ContractError;

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "wyndex-pair-lsd";
/// Contract version that is used for migration.
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const WEEK: u64 = 7 * 24 * 60 * 60;

/// Creates a new contract with the specified parameters in [`InstantiateMsg`].
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let asset_infos = check_asset_infos(deps.api, &msg.asset_infos)?;

    // Only 2 assets makes sense for lsd (Asset and Asset-LSD)
    if asset_infos.len() != 2 {
        return Err(ContractError::InvalidNumberOfAssets { min: 2, max: 2 });
    }

    if msg.init_params.is_none() {
        return Err(ContractError::InitParamsNotFound {});
    }

    msg.validate_fees()?;

    let params: StablePoolParams = from_binary(&msg.init_params.unwrap())?;

    if params.amp == 0 || params.amp > MAX_AMP {
        return Err(ContractError::IncorrectAmp { max_amp: MAX_AMP });
    }

    let lsd_data: Option<LsdData> = if let Some(info) = params.lsd {
        ensure!(
            info.target_rate_epoch <= WEEK,
            ContractError::InvalidTargetRateEpoch {}
        );
        Some(LsdData {
            asset: info.asset.validate(deps.api)?,
            lsd_hub: deps.api.addr_validate(&info.hub)?,
            target_rate: Decimal::one(),
            target_rate_epoch: info.target_rate_epoch,
            last_target_query: 0,
        })
    } else {
        None
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let greatest_precision = store_precisions(deps.branch(), &asset_infos)?;

    // Initializing cumulative prices
    let mut cumulative_prices = vec![];
    for from_pool in &asset_infos {
        for to_pool in &asset_infos {
            if !from_pool.eq(to_pool) {
                cumulative_prices.push((from_pool.clone(), to_pool.clone(), Uint128::zero()))
            }
        }
    }

    let factory_addr = deps.api.addr_validate(msg.factory_addr.as_str())?;

    let create_lp_token_msg = create_lp_token(
        &deps.querier,
        &env,
        msg.token_code_id,
        &asset_infos,
        &factory_addr,
    )?;

    let config = Config {
        owner: addr_opt_validate(deps.api, &params.owner)?,
        pair_info: PairInfo {
            contract_addr: env.contract.address.clone(),
            liquidity_token: Addr::unchecked(""),
            staking_addr: Addr::unchecked(""),
            asset_infos,
            pair_type: PairType::Lsd {},
            fee_config: msg.fee_config,
        },
        factory_addr,
        block_time_last: 0,
        init_amp: params.amp * AMP_PRECISION,
        init_amp_time: env.block.time.seconds(),
        next_amp: params.amp * AMP_PRECISION,
        next_amp_time: env.block.time.seconds(),
        greatest_precision,
        cumulative_prices,
        trading_starts: msg.trading_starts,
        lsd: lsd_data,
    };

    CONFIG.save(deps.storage, &config)?;
    FROZEN.save(deps.storage, &false)?;

    save_tmp_staking_config(deps.storage, &msg.staking_config)?;

    Ok(Response::new().add_submessage(create_lp_token_msg))
}

/// The entry point to the contract for processing replies from submessages.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let res = handle_reply(&deps, msg, &config.factory_addr, &mut config.pair_info)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(res)
}

/// Manages the contract migration.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    match msg {
        MigrateMsg::UpdateFreeze {
            frozen,
            circuit_breaker,
        } => {
            FROZEN.save(deps.storage, &frozen)?;
            if let Some(circuit_breaker) = circuit_breaker {
                CIRCUIT_BREAKER.save(deps.storage, &deps.api.addr_validate(&circuit_breaker)?)?;
            }
        }
    }

    Ok(Response::new())
}

/// Helper function to check if the pool has been frozen
fn check_if_frozen(deps: &DepsMut) -> Result<(), ContractError> {
    let is_frozen: bool = FROZEN.load(deps.storage)?;
    ensure!(!is_frozen, ContractError::ContractFrozen {});
    Ok(())
}

/// Exposes all the execute functions available in the contract.
///
/// ## Variants
/// * **ExecuteMsg::UpdateConfig { params: Binary }** Updates the contract configuration with the specified
/// input parameters.
///
/// * **ExecuteMsg::Receive(msg)** Receives a message of type [`Cw20ReceiveMsg`] and processes
/// it depending on the received template.
///
/// * **ExecuteMsg::ProvideLiquidity {
///             assets,
///             slippage_tolerance,
///             receiver,
///         }** Provides liquidity in the pair using the specified input parameters.
///
/// * **ExecuteMsg::Swap {
///             offer_asset,
///             belief_price,
///             max_spread,
///             to,
///         }** Performs an swap using the specified parameters.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    if migration_check(deps.querier, &cfg.factory_addr, &env.contract.address)? {
        return Err(ContractError::PairIsNotMigrated {});
    }

    match msg {
        ExecuteMsg::UpdateConfig { params } => update_config(deps, env, info, params),
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::ProvideLiquidity {
            assets,
            receiver,
            slippage_tolerance: _,
        } => provide_liquidity(deps, env, info, assets, receiver),
        ExecuteMsg::UpdateFees { fee_config } => update_fees(deps, info, fee_config),
        ExecuteMsg::Swap {
            offer_asset,
            ask_asset_info,
            belief_price,
            max_spread,
            to,
            referral_address,
            referral_commission,
        } => {
            let offer_asset = offer_asset.validate(deps.api)?;
            if !offer_asset.is_native_token() {
                return Err(ContractError::Unauthorized {});
            }

            let to_addr = addr_opt_validate(deps.api, &to)?;
            let referral_address = addr_opt_validate(deps.api, &referral_address)?;

            swap(
                deps,
                env,
                info.clone(),
                info.sender,
                offer_asset,
                ask_asset_info,
                belief_price,
                max_spread,
                to_addr,
                referral_address,
                referral_commission,
            )
        }
        ExecuteMsg::ProposeNewOwner { owner, expires_in } => {
            let config = CONFIG.load(deps.storage)?;
            let factory_config = query_factory_config(&deps.querier, config.factory_addr)?;

            propose_new_owner(
                deps,
                info,
                env,
                owner,
                expires_in,
                config.owner.unwrap_or(factory_config.owner),
                OWNERSHIP_PROPOSAL,
            )
            .map_err(|e| e.into())
        }
        ExecuteMsg::DropOwnershipProposal {} => {
            let config = CONFIG.load(deps.storage)?;
            let factory_config = query_factory_config(&deps.querier, config.factory_addr)?;

            drop_ownership_proposal(
                deps,
                info,
                config.owner.unwrap_or(factory_config.owner),
                OWNERSHIP_PROPOSAL,
            )
            .map_err(|e| e.into())
        }
        ExecuteMsg::ClaimOwnership {} => {
            claim_ownership(deps, info, env, OWNERSHIP_PROPOSAL, |deps, new_owner| {
                CONFIG.update::<_, StdError>(deps.storage, |mut config| {
                    config.owner = Some(new_owner);
                    Ok(config)
                })?;

                Ok(())
            })
            .map_err(|e| e.into())
        }
        ExecuteMsg::Freeze { frozen } => {
            ensure!(
                info.sender
                    == CIRCUIT_BREAKER
                        .may_load(deps.storage)?
                        .unwrap_or_else(|| Addr::unchecked("")),
                ContractError::Unauthorized {}
            );
            FROZEN.save(deps.storage, &frozen)?;
            Ok(Response::new())
        }
    }
}

/// Receives a message of type [`Cw20ReceiveMsg`] and processes it depending on the received template.
///
/// * **cw20_msg** is the CW20 receive message to process.
pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::Swap {
            ask_asset_info,
            belief_price,
            max_spread,
            to,
            referral_address,
            referral_commission,
        } => {
            // Only asset contract can execute this message
            check_cw20_in_pool(
                &CONFIG.load(deps.storage)?.pair_info.asset_infos,
                &info.sender,
            )?;

            let to_addr = addr_opt_validate(deps.api, &to)?;
            let referral_address = addr_opt_validate(deps.api, &referral_address)?;
            let contract_addr = info.sender.clone();
            let sender = deps.api.addr_validate(&cw20_msg.sender)?;
            swap(
                deps,
                env,
                info,
                sender,
                AssetValidated {
                    info: AssetInfoValidated::Token(contract_addr),
                    amount: cw20_msg.amount,
                },
                ask_asset_info,
                belief_price,
                max_spread,
                to_addr,
                referral_address,
                referral_commission,
            )
        }
        Cw20HookMsg::WithdrawLiquidity { assets } => {
            let sender = deps.api.addr_validate(&cw20_msg.sender)?;
            withdraw_liquidity(deps, env, info, sender, cw20_msg.amount, assets)
        }
    }
}

pub fn update_fees(
    deps: DepsMut,
    info: MessageInfo,
    fee_config: FeeConfig,
) -> Result<Response, ContractError> {
    check_if_frozen(&deps)?;

    let mut config = CONFIG.load(deps.storage)?;

    // check permissions
    if Some(info.sender) != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // update config
    config.pair_info.fee_config = fee_config;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

/// Provides liquidity with the specified input parameters.
///
/// * **assets** vector with assets available in the pool.
///
///
/// * **receiver** address that receives LP tokens. If this address isn't specified, the function will default to the caller.
///
/// NOTE - the address that wants to provide liquidity should approve the pair contract to pull its relevant tokens.
pub fn provide_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    assets: Vec<Asset>,
    receiver: Option<String>,
) -> Result<Response, ContractError> {
    check_if_frozen(&deps)?;
    let assets = check_assets(deps.api, &assets)?;
    let mut config = CONFIG.load(deps.storage)?;

    if assets.len() > config.pair_info.asset_infos.len() {
        return Err(ContractError::TooManyAssets {
            max: config.pair_info.asset_infos.len(),
            provided: assets.len(),
        });
    }

    let save_config = update_target_rate(deps.querier, &mut config, &env)?;

    let pools: HashMap<_, _> = config
        .pair_info
        .query_pools(&deps.querier, &env.contract.address)?
        .into_iter()
        .map(|pool| (pool.info, pool.amount))
        .collect();

    let mut non_zero_flag = false;

    let mut assets_collection = assets
        .clone()
        .into_iter()
        .map(|asset| {
            asset.assert_sent_native_token_balance(&info)?;

            // Check that at least one asset is non-zero
            if !asset.amount.is_zero() {
                non_zero_flag = true;
            }

            // Get appropriate pool
            let pool = pools
                .get(&asset.info)
                .copied()
                .ok_or_else(|| ContractError::InvalidAsset(asset.info.to_string()))?;

            Ok((asset, pool))
        })
        .collect::<Result<Vec<_>, ContractError>>()?;

    // If some assets are omitted then add them explicitly with 0 deposit
    pools.iter().for_each(|(pool_info, pool_amount)| {
        if !assets.iter().any(|asset| asset.info.eq(pool_info)) {
            assets_collection.push((
                AssetValidated {
                    amount: Uint128::zero(),
                    info: pool_info.clone(),
                },
                *pool_amount,
            ));
        }
    });

    if !non_zero_flag {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let mut messages = vec![];
    for (deposit, pool) in assets_collection.iter_mut() {
        // We cannot put a zero amount into an empty pool.
        if deposit.amount.is_zero() && pool.is_zero() {
            return Err(ContractError::InvalidProvideLPsWithSingleToken {});
        }

        // Transfer only non-zero amount
        if !deposit.amount.is_zero() {
            // If the pool is a token contract, then we need to execute a TransferFrom msg to receive funds
            if let AssetInfoValidated::Token(contract_addr) = &deposit.info {
                messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: contract_addr.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                        owner: info.sender.to_string(),
                        recipient: env.contract.address.to_string(),
                        amount: deposit.amount,
                    })?,
                    funds: vec![],
                }))
            } else {
                // If the asset is a native token, the pool balance already increased
                // To calculate the pool balance properly, we should subtract the user deposit from the recorded pool token amount
                *pool = pool.checked_sub(deposit.amount)?;
            }
        }
    }

    let assets_collection = assets_collection
        .iter()
        .cloned()
        .map(|(asset, pool)| {
            let coin_precision = get_precision(deps.storage, &asset.info)?;
            Ok((
                asset.to_decimal_asset(coin_precision)?,
                Decimal256::with_precision(pool, coin_precision)?,
            ))
        })
        .collect::<StdResult<Vec<(DecimalAsset, Decimal256)>>>()?;

    let n_coins = config.pair_info.asset_infos.len() as u8;

    let amp = compute_current_amp(&config, &env)?;

    // Initial invariant (D)
    let old_balances = assets_collection
        .iter()
        .map(|(_, pool)| *pool)
        .collect_vec();
    let init_d = compute_d(amp, &old_balances, config.greatest_precision)?;

    // Invariant (D) after deposit added
    let mut new_balances: Vec<_> = assets_collection
        .iter()
        .map(|(deposit, pool)| Ok(pool + deposit.amount))
        .collect::<StdResult<Vec<_>>>()?;
    let deposit_d = compute_d(amp, &new_balances, config.greatest_precision)?;

    let total_share = query_supply(&deps.querier, &config.pair_info.liquidity_token)?;
    let share = if total_share.is_zero() {
        let share = deposit_d
            .to_uint128_with_precision(config.greatest_precision)?
            .checked_sub(MINIMUM_LIQUIDITY_AMOUNT)
            .map_err(|_| ContractError::MinimumLiquidityAmountError {})?;

        messages.extend(mint_token_message(
            &config.pair_info.liquidity_token,
            &env.contract.address,
            MINIMUM_LIQUIDITY_AMOUNT,
        )?);

        // share cannot become zero after minimum liquidity subtraction
        if share.is_zero() {
            return Err(ContractError::MinimumLiquidityAmountError {});
        }

        share
    } else {
        // Get fee info from the factory
        let fee_info = query_fee_info(
            &deps.querier,
            &config.factory_addr,
            config.pair_info.pair_type.clone(),
        )?;

        // total_fee_rate * N_COINS / (4 * (N_COINS - 1))
        let fee = fee_info
            .total_fee_rate
            .checked_mul(Decimal::from_ratio(n_coins, 4 * (n_coins - 1)))?;

        let fee = Decimal256::new(fee.atomics().into());

        for i in 0..n_coins as usize {
            let ideal_balance = deposit_d.checked_multiply_ratio(old_balances[i], init_d)?;
            let difference = if ideal_balance > new_balances[i] {
                ideal_balance - new_balances[i]
            } else {
                new_balances[i] - ideal_balance
            };
            // Fee will be charged only during imbalanced provide i.e. if invariant D was changed
            new_balances[i] -= fee.checked_mul(difference)?;
        }

        let after_fee_d = compute_d(amp, &new_balances, config.greatest_precision)?;

        let share = Decimal256::with_precision(total_share, config.greatest_precision)?
            .checked_multiply_ratio(after_fee_d.saturating_sub(init_d), init_d)?
            .to_uint128_with_precision(config.greatest_precision)?;

        if share.is_zero() {
            return Err(ContractError::LiquidityAmountTooSmall {});
        }

        share
    };

    // Mint LP token for the caller (or for the receiver if it was set)
    let receiver = addr_opt_validate(deps.api, &receiver)?.unwrap_or_else(|| info.sender.clone());
    messages.extend(mint_token_message(
        &config.pair_info.liquidity_token,
        &receiver,
        share,
    )?);

    // using assets_collection, since the deposit amount is already subtracted there
    let old_pools = assets_collection
        .iter()
        .map(|(a, p)| DecimalAsset {
            info: a.info.clone(),
            amount: *p,
        })
        .collect::<Vec<_>>();

    // calculate pools with deposited balances
    let new_pools = assets_collection
        .into_iter()
        .map(|(mut asset, pool)| {
            // add deposit amount back to pool amount, so we can calculate the new price
            asset.amount += pool;
            asset
        })
        .collect::<Vec<_>>();
    let new_price = calc_new_price_a_per_b(deps.as_ref(), &env, &config, &new_pools)?;

    if total_share.is_zero() {
        // initialize oracle storage
        wyndex::oracle::initialize_oracle(deps.storage, &env, new_price)?;
    } else {
        wyndex::oracle::store_oracle_price(deps.storage, &env, new_price)?;
    }

    if accumulate_prices(deps.as_ref(), &env, &mut config, &old_pools)? || save_config {
        CONFIG.save(deps.storage, &config)?;
    }

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        attr("action", "provide_liquidity"),
        attr("sender", info.sender),
        attr("receiver", receiver),
        attr("assets", assets.iter().join(", ")),
        attr("share", share),
    ]))
}

/// Withdraw liquidity from the pool.
/// * **sender** is the address that will receive assets back from the pair contract.
///
/// * **amount** is the amount of LP tokens to burn.
///
/// * **assets** optional array which specifies the assets amount to withdraw.
pub fn withdraw_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: Addr,
    amount: Uint128,
    assets: Vec<Asset>,
) -> Result<Response, ContractError> {
    let assets = check_assets(deps.api, &assets)?;
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.pair_info.liquidity_token {
        return Err(ContractError::Unauthorized {});
    }

    let burn_amount;
    let refund_assets;
    let mut messages = vec![];

    let (pools, total_share) = pool_info(deps.querier, &config)?;
    if assets.is_empty() {
        burn_amount = amount;
        refund_assets = get_share_in_assets(&pools, amount, total_share);
    } else {
        // Imbalanced withdraw
        burn_amount = imbalanced_withdraw(deps.as_ref(), &env, &config, amount, &assets)?;
        if burn_amount < amount {
            // Returning unused LP tokens back to the user
            messages.push(
                wasm_execute(
                    &config.pair_info.liquidity_token,
                    &Cw20ExecuteMsg::Transfer {
                        recipient: sender.to_string(),
                        amount: amount - burn_amount,
                    },
                    vec![],
                )?
                .into(),
            )
        }
        refund_assets = assets;
    }

    messages.extend(
        refund_assets
            .clone()
            .into_iter()
            .map(|asset| asset.into_msg(&sender))
            .collect::<StdResult<Vec<_>>>()?,
    );
    messages.push(
        wasm_execute(
            &config.pair_info.liquidity_token,
            &Cw20ExecuteMsg::Burn {
                amount: burn_amount,
            },
            vec![],
        )?
        .into(),
    );

    let old_pools = pools
        .iter()
        .map(|pool| {
            let precision = get_precision(deps.storage, &pool.info)?;
            pool.to_decimal_asset(precision)
        })
        .collect::<StdResult<Vec<DecimalAsset>>>()?;

    let save_config = update_target_rate(deps.querier, &mut config, &env)?;

    // calculate pools with withdrawn balances
    let new_pools = pools
        .into_iter()
        .zip(refund_assets.iter())
        .map(|(mut pool, refund)| {
            pool.amount -= refund.amount;
            let precision = get_precision(deps.storage, &pool.info)?;
            pool.to_decimal_asset(precision)
        })
        .collect::<StdResult<Vec<DecimalAsset>>>()?;
    let new_price = calc_new_price_a_per_b(deps.as_ref(), &env, &config, &new_pools)?;
    wyndex::oracle::store_oracle_price(deps.storage, &env, new_price)?;

    if accumulate_prices(deps.as_ref(), &env, &mut config, &old_pools)? || save_config {
        CONFIG.save(deps.storage, &config)?;
    }

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        attr("action", "withdraw_liquidity"),
        attr("sender", sender),
        attr("withdrawn_share", amount),
        attr("refund_assets", refund_assets.iter().join(", ")),
    ]))
}

/// Imbalanced withdraw liquidity from the pool. Returns a [`ContractError`] on failure,
/// otherwise returns the number of LP tokens to burn.
///
/// * **provided_amount** amount of provided LP tokens to withdraw liquidity with.
///
/// * **assets** specifies the assets amount to withdraw.
fn imbalanced_withdraw(
    deps: Deps,
    env: &Env,
    config: &Config,
    provided_amount: Uint128,
    assets: &[AssetValidated],
) -> Result<Uint128, ContractError> {
    if assets.len() > config.pair_info.asset_infos.len() {
        return Err(ContractError::TooManyAssets {
            max: config.pair_info.asset_infos.len(),
            provided: assets.len(),
        });
    }

    let pools: HashMap<_, _> = config
        .pair_info
        .query_pools(&deps.querier, &env.contract.address)?
        .into_iter()
        .map(|pool| (pool.info, pool.amount))
        .collect();

    let mut assets_collection = assets
        .iter()
        .cloned()
        .map(|asset| {
            let precision = get_precision(deps.storage, &asset.info)?;
            // Get appropriate pool
            let pool = pools
                .get(&asset.info)
                .copied()
                .ok_or_else(|| ContractError::InvalidAsset(asset.info.to_string()))?;

            Ok((
                asset.to_decimal_asset(precision)?,
                Decimal256::with_precision(pool, precision)?,
            ))
        })
        .collect::<Result<Vec<_>, ContractError>>()?;

    // If some assets are omitted then add them explicitly with 0 withdraw amount
    pools
        .into_iter()
        .try_for_each(|(pool_info, pool_amount)| -> StdResult<()> {
            if !assets.iter().any(|asset| asset.info == pool_info) {
                let precision = get_precision(deps.storage, &pool_info)?;

                assets_collection.push((
                    DecimalAsset {
                        amount: Decimal256::zero(),
                        info: pool_info,
                    },
                    Decimal256::with_precision(pool_amount, precision)?,
                ));
            }
            Ok(())
        })?;

    let n_coins = config.pair_info.asset_infos.len() as u8;

    let amp = compute_current_amp(config, env)?;

    // Initial invariant (D)
    let old_balances = assets_collection
        .iter()
        .map(|(_, pool)| *pool)
        .collect_vec();
    let init_d = compute_d(amp, &old_balances, config.greatest_precision)?;

    // Invariant (D) after assets withdrawn
    let mut new_balances = assets_collection
        .iter()
        .cloned()
        .map(|(withdraw, pool)| Ok(pool - withdraw.amount))
        .collect::<StdResult<Vec<Decimal256>>>()?;
    let withdraw_d = compute_d(amp, &new_balances, config.greatest_precision)?;

    // Get fee info from the factory
    let fee_info = query_fee_info(
        &deps.querier,
        &config.factory_addr,
        config.pair_info.pair_type.clone(),
    )?;

    // total_fee_rate * N_COINS / (4 * (N_COINS - 1))
    let fee = fee_info
        .total_fee_rate
        .checked_mul(Decimal::from_ratio(n_coins, 4 * (n_coins - 1)))?;

    let fee = Decimal256::new(fee.atomics().into());

    for i in 0..n_coins as usize {
        let ideal_balance = withdraw_d.checked_multiply_ratio(old_balances[i], init_d)?;
        let difference = if ideal_balance > new_balances[i] {
            ideal_balance - new_balances[i]
        } else {
            new_balances[i] - ideal_balance
        };
        new_balances[i] -= fee.checked_mul(difference)?;
    }

    let after_fee_d = compute_d(amp, &new_balances, config.greatest_precision)?;

    let total_share = Uint256::from(query_supply(
        &deps.querier,
        &config.pair_info.liquidity_token,
    )?);
    // How many tokens do we need to burn to withdraw asked assets?
    let burn_amount = total_share
        .checked_multiply_ratio(
            init_d.atomics().checked_sub(after_fee_d.atomics())?,
            init_d.atomics(),
        )?
        .checked_add(Uint256::from(1u8))?; // In case of rounding errors - make it unfavorable for the "attacker"

    let burn_amount = burn_amount.try_into()?;

    if burn_amount > provided_amount {
        return Err(StdError::generic_err(format!(
            "Not enough LP tokens. You need {} LP tokens.",
            burn_amount
        ))
        .into());
    }

    Ok(burn_amount)
}

/// Performs an swap operation with the specified parameters.
///
/// * **sender** is the sender of the swap operation.
///
/// * **offer_asset** proposed asset for swapping.
///
/// * **belief_price** is used to calculate the maximum swap spread.
///
/// * **max_spread** sets the maximum spread of the swap operation.
///
/// * **to** sets the recipient of the swap operation.
///
/// NOTE - the address that wants to swap should approve the pair contract to pull the offer token.
#[allow(clippy::too_many_arguments)]
pub fn swap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: Addr,
    mut offer_asset: AssetValidated,
    ask_asset_info: Option<AssetInfo>,
    belief_price: Option<Decimal>,
    max_spread: Option<Decimal>,
    to: Option<Addr>,
    referral_address: Option<Addr>,
    referral_commission: Option<Decimal>,
) -> Result<Response, ContractError> {
    check_if_frozen(&deps)?;
    offer_asset.assert_sent_native_token_balance(&info)?;

    let ask_asset_info = ask_asset_info.map(|a| a.validate(deps.api)).transpose()?;
    let mut config = CONFIG.load(deps.storage)?;

    if env.block.time.seconds() < config.trading_starts {
        return Err(ContractError::TradingNotStarted {});
    }

    // If the asset balance already increased
    // We should subtract the user deposit from the pool offer asset amount
    let pools = config
        .pair_info
        .query_pools(&deps.querier, &env.contract.address)?
        .into_iter()
        .map(|mut pool| {
            if pool.info.equal(&offer_asset.info) {
                pool.amount = pool.amount.checked_sub(offer_asset.amount)?;
            }
            let token_precision = get_precision(deps.storage, &pool.info)?;
            Ok(DecimalAsset {
                info: pool.info,
                amount: Decimal256::with_precision(pool.amount, token_precision)?,
            })
        })
        .collect::<StdResult<Vec<_>>>()?;

    let mut messages = Vec::new();

    // Get config from the factory
    let factory_config = query_factory_config(&deps.querier, &config.factory_addr)?;

    handle_referral(
        &factory_config,
        referral_address,
        referral_commission,
        &mut offer_asset,
        &mut messages,
    )?;

    let (offer_pool, ask_pool) =
        select_pools(Some(&offer_asset.info), ask_asset_info.as_ref(), &pools)?;

    let offer_precision = get_precision(deps.storage, &offer_pool.info)?;

    // Check if the liquidity is non-zero
    check_swap_parameters(
        pools
            .iter()
            .map(|pool| {
                pool.amount
                    .to_uint128_with_precision(get_precision(deps.storage, &pool.info)?)
            })
            .collect::<StdResult<Vec<Uint128>>>()?,
        offer_asset.amount,
    )?;

    let save_config = update_target_rate(deps.querier, &mut config, &env)?;
    let SwapResult {
        return_amount,
        spread_amount,
    } = compute_swap(
        deps.storage,
        &env,
        &config,
        &offer_asset.to_decimal_asset(offer_precision)?,
        &offer_pool,
        &ask_pool,
        &pools,
    )?;

    let commission_amount = config
        .pair_info
        .fee_config
        .total_fee_rate()
        .checked_mul_uint128(return_amount)?;
    let return_amount = return_amount.saturating_sub(commission_amount);

    // Check the max spread limit (if it was specified)
    assert_max_spread(
        belief_price,
        max_spread,
        offer_asset.amount,
        return_amount,
        spread_amount + commission_amount,
    )?;

    let receiver = to.unwrap_or_else(|| sender.clone());

    messages.push(
        AssetValidated {
            info: ask_pool.info.clone(),
            amount: return_amount,
        }
        .into_msg(&receiver)?,
    );

    // Compute the protocol fee
    let mut protocol_fee_amount = Uint128::zero();
    if let Some(fee_address) = factory_config.fee_address {
        if let Some(f) = calculate_protocol_fee(
            &ask_pool.info,
            commission_amount,
            config.pair_info.fee_config.protocol_fee_rate(),
        ) {
            protocol_fee_amount = f.amount;
            messages.push(f.into_msg(fee_address)?);
        }
    }

    // calculate pools with deposited / withdrawn balances
    let new_pools = pools
        .iter()
        .cloned()
        .map(|mut pool| -> StdResult<DecimalAsset> {
            if pool.info.equal(&offer_asset.info) {
                // add offer amount to pool (it was already subtracted right at the beginning)
                pool.amount = pool.amount.checked_add(Decimal256::with_precision(
                    offer_asset.amount,
                    offer_precision,
                )?)?;
            } else if pool.info.equal(&ask_pool.info) {
                // subtract fee and return amount from ask pool
                let ask_precision = get_precision(deps.storage, &ask_pool.info)?;
                pool.amount = pool.amount.checked_sub(Decimal256::with_precision(
                    return_amount + protocol_fee_amount,
                    ask_precision,
                )?)?;
            }
            Ok(pool)
        })
        .collect::<StdResult<Vec<_>>>()?;
    let new_price = calc_new_price_a_per_b(deps.as_ref(), &env, &config, &new_pools)?;
    wyndex::oracle::store_oracle_price(deps.storage, &env, new_price)?;

    if accumulate_prices(deps.as_ref(), &env, &mut config, &pools)? || save_config {
        CONFIG.save(deps.storage, &config)?;
    }

    Ok(Response::new()
        .add_messages(
            // 1. send collateral tokens from the contract to a user
            // 2. send inactive commission fees to the protocol
            messages,
        )
        .add_attributes(vec![
            attr("action", "swap"),
            attr("sender", sender),
            attr("receiver", receiver),
            attr("offer_asset", offer_asset.info.to_string()),
            attr("ask_asset", ask_pool.info.to_string()),
            attr("offer_amount", offer_asset.amount),
            attr("return_amount", return_amount),
            attr("spread_amount", spread_amount),
            attr("commission_amount", commission_amount),
            attr("protocol_fee_amount", protocol_fee_amount),
        ]))
}

/// Calculates the amount of fees the protocol gets according to specified pair parameters.
/// Returns a [`None`] if the protocol fee is zero, otherwise returns a [`Asset`] struct with the specified attributes.
///
/// * **pool_info** contains information about the pool asset for which the commission will be calculated.
///
/// * **commission_amount** is the total amount of fees charged for a swap.
///
/// * **protocol_commission_rate** is the percentage of fees that go to the protocol.
pub fn calculate_protocol_fee(
    pool_info: &AssetInfoValidated,
    commission_amount: Uint128,
    protocol_commission_rate: Decimal,
) -> Option<AssetValidated> {
    let protocol_fee: Uint128 = commission_amount * protocol_commission_rate;
    if protocol_fee.is_zero() {
        return None;
    }

    Some(AssetValidated {
        info: pool_info.clone(),
        amount: protocol_fee,
    })
}

/// Exposes all the queries available in the contract.
/// ## Queries
/// * **QueryMsg::Pair {}** Returns information about the pair in an object of type [`PairInfo`].
///
/// * **QueryMsg::Pool {}** Returns information about the amount of assets in the pair contract as
/// well as the amount of LP tokens issued using an object of type [`PoolResponse`].
///
/// * **QueryMsg::Share { amount }** Returns the amount of assets that could be withdrawn from the pool
/// using a specific amount of LP tokens. The result is returned in a vector that contains objects of type [`Asset`].
///
/// * **QueryMsg::Simulation { offer_asset }** Returns the result of a swap simulation using a [`SimulationResponse`] object.
///
/// * **QueryMsg::ReverseSimulation { ask_asset }** Returns the result of a reverse swap simulation using
/// a [`ReverseSimulationResponse`] object.
///
/// * **QueryMsg::CumulativePrices {}** Returns information about cumulative prices for the assets in the
/// pool using a [`CumulativePricesResponse`] object.
///
/// * **QueryMsg::HistoricalPrices { duration }** Returns historical price information for the assets in the
/// pool using a [`HistoricalPricesResponse`] object.
///
/// * **QueryMsg::Config {}** Returns the configuration for the pair contract using a [`ConfigResponse`] object.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Pair {} => to_binary(&CONFIG.load(deps.storage)?.pair_info),
        QueryMsg::Pool {} => to_binary(&query_pool(deps)?),
        QueryMsg::Share { amount } => to_binary(&query_share(deps, amount)?),
        QueryMsg::Simulation {
            offer_asset,
            ask_asset_info,
            referral,
            referral_commission,
        } => to_binary(&query_simulation(
            deps,
            env,
            offer_asset,
            ask_asset_info,
            referral,
            referral_commission,
        )?),
        QueryMsg::ReverseSimulation {
            offer_asset_info,
            ask_asset,
            referral,
            referral_commission,
        } => to_binary(&query_reverse_simulation(
            deps,
            env,
            ask_asset,
            offer_asset_info,
            referral,
            referral_commission,
        )?),
        QueryMsg::CumulativePrices {} => to_binary(&query_cumulative_prices(deps, env)?),
        QueryMsg::Twap {
            duration,
            start_age,
            end_age,
        } => to_binary(&wyndex::oracle::query_oracle_range(
            deps.storage,
            &env,
            &CONFIG.load(deps.storage)?.pair_info.asset_infos,
            duration,
            start_age,
            end_age,
        )?),
        QueryMsg::Config {} => to_binary(&query_config(deps, env)?),
        QueryMsg::QueryComputeD {} => to_binary(&query_compute_d(deps, env)?),
        QueryMsg::SpotPrice { offer, ask } => to_binary(&query_spot_price(deps, env, offer, ask)?),
        QueryMsg::SpotPricePrediction {
            offer,
            ask,
            max_trade,
            target_price,
            iterations,
        } => to_binary(&query_spot_price_prediction(
            deps,
            env,
            offer,
            ask,
            max_trade,
            target_price,
            iterations,
        )?),
    }
}

/// Returns the amounts of assets in the pair contract as well as the amount of LP
/// tokens currently minted in an object of type [`PoolResponse`].
pub fn query_pool(deps: Deps) -> StdResult<PoolResponse> {
    let config = CONFIG.load(deps.storage)?;
    let (assets, total_share) = pool_info(deps.querier, &config)?;

    let resp = PoolResponse {
        assets,
        total_share,
    };

    Ok(resp)
}

/// Returns the amount of assets that could be withdrawn from the pool using a specific amount of LP tokens.
/// The result is returned in a vector that contains objects of type [`Asset`].
///
/// * **amount** is the amount of LP tokens for which we calculate associated amounts of assets.
pub fn query_share(deps: Deps, amount: Uint128) -> StdResult<Vec<AssetValidated>> {
    let config = CONFIG.load(deps.storage)?;
    let (pools, total_share) = pool_info(deps.querier, &config)?;
    let refund_assets = get_share_in_assets(&pools, amount, total_share);

    Ok(refund_assets)
}

/// Returns information about a swap simulation in a [`SimulationResponse`] object.
///
/// * **offer_asset** is the asset to swap as well as an amount of the said asset.
pub fn query_simulation(
    deps: Deps,
    env: Env,
    offer_asset: Asset,
    ask_asset_info: Option<AssetInfo>,
    referral: bool,
    referral_commission: Option<Decimal>,
) -> StdResult<SimulationResponse> {
    let mut offer_asset = offer_asset.validate(deps.api)?;
    let ask_asset_info = ask_asset_info.map(|a| a.validate(deps.api)).transpose()?;
    let mut config = CONFIG.load(deps.storage)?;
    let pools = config
        .pair_info
        .query_pools_decimal(&deps.querier, &config.pair_info.contract_addr)?;

    let referral_amount = if referral {
        let factory_config = query_factory_config(&deps.querier, &config.factory_addr)?;
        take_referral(&factory_config, referral_commission, &mut offer_asset)?
    } else {
        Uint128::zero()
    };

    let (offer_pool, ask_pool) =
        select_pools(Some(&offer_asset.info), ask_asset_info.as_ref(), &pools)
            .map_err(|err| StdError::generic_err(format!("{err}")))?;

    let offer_precision = get_precision(deps.storage, &offer_pool.info)?;

    if check_swap_parameters(
        pools
            .iter()
            .map(|pool| {
                pool.amount
                    .to_uint128_with_precision(get_precision(deps.storage, &pool.info)?)
            })
            .collect::<StdResult<Vec<Uint128>>>()?,
        offer_asset.amount,
    )
    .is_err()
    {
        return Ok(SimulationResponse {
            return_amount: Uint128::zero(),
            spread_amount: Uint128::zero(),
            commission_amount: Uint128::zero(),
            referral_amount: Uint128::zero(),
        });
    }

    update_target_rate(deps.querier, &mut config, &env)?;
    let SwapResult {
        return_amount,
        spread_amount,
    } = compute_swap(
        deps.storage,
        &env,
        &config,
        &offer_asset.to_decimal_asset(offer_precision)?,
        &offer_pool,
        &ask_pool,
        &pools,
    )
    .map_err(|err| StdError::generic_err(format!("{err}")))?;

    let commission_amount = config
        .pair_info
        .fee_config
        .total_fee_rate()
        .checked_mul_uint128(return_amount)?;
    let return_amount = return_amount.saturating_sub(commission_amount);

    Ok(SimulationResponse {
        return_amount,
        spread_amount,
        commission_amount,
        referral_amount,
    })
}

/// Returns information about a reverse swap simulation in a [`ReverseSimulationResponse`] object.
///
/// * **ask_asset** is the asset to swap to as well as the desired amount of ask
/// assets to receive from the swap.
///
/// * **offer_asset_info** is optional field which specifies the asset to swap from.
/// May be omitted only in case the pool length is 2.
pub fn query_reverse_simulation(
    deps: Deps,
    env: Env,
    ask_asset: Asset,
    offer_asset_info: Option<AssetInfo>,
    referral: bool,
    referral_commission: Option<Decimal>,
) -> StdResult<ReverseSimulationResponse> {
    let ask_asset = ask_asset.validate(deps.api)?;
    let offer_asset_info = offer_asset_info.map(|a| a.validate(deps.api)).transpose()?;

    let mut config = CONFIG.load(deps.storage)?;
    let pools = config
        .pair_info
        .query_pools_decimal(&deps.querier, &config.pair_info.contract_addr)?;
    let (offer_pool, ask_pool) =
        select_pools(offer_asset_info.as_ref(), Some(&ask_asset.info), &pools)
            .map_err(|err| StdError::generic_err(format!("{err}")))?;

    let offer_precision = get_precision(deps.storage, &offer_pool.info)?;
    let ask_precision = get_precision(deps.storage, &ask_asset.info)?;

    // Check the swap parameters are valid
    if check_swap_parameters(
        pools
            .iter()
            .map(|pool| {
                pool.amount
                    .to_uint128_with_precision(get_precision(deps.storage, &pool.info)?)
            })
            .collect::<StdResult<Vec<Uint128>>>()?,
        ask_asset.amount,
    )
    .is_err()
    {
        return Ok(ReverseSimulationResponse {
            offer_amount: Uint128::zero(),
            spread_amount: Uint128::zero(),
            commission_amount: Uint128::zero(),
            referral_amount: Uint128::zero(),
        });
    }

    // Get fee info from factory
    let fee_info = query_fee_info(
        &deps.querier,
        &config.factory_addr,
        config.pair_info.pair_type.clone(),
    )?;
    let before_commission = (Decimal256::one()
        - Decimal256::new(fee_info.total_fee_rate.atomics().into()))
    .inv()
    .unwrap_or_else(Decimal256::one)
    .checked_mul(Decimal256::with_precision(ask_asset.amount, ask_precision)?)?;

    update_target_rate(deps.querier, &mut config, &env)?;
    let new_offer_pool_amount = calc_y(
        &ask_pool,
        &offer_pool.info,
        ask_pool.amount - before_commission,
        &pools,
        compute_current_amp(&config, &env)?,
        config.greatest_precision,
        &config,
    )?;

    let offer_amount = new_offer_pool_amount.checked_sub(
        offer_pool
            .amount
            .to_uint128_with_precision(config.greatest_precision)?,
    )?;
    let offer_amount = adjust_precision(offer_amount, config.greatest_precision, offer_precision)?;

    // `offer_pool.info` is already validated
    let offer_asset = AssetValidated {
        info: offer_pool.info,
        amount: offer_amount,
    };
    let (offer_asset, referral_amount) = add_referral(
        &deps.querier,
        &config.factory_addr,
        referral,
        referral_commission,
        offer_asset,
    )?;

    Ok(ReverseSimulationResponse {
        offer_amount: offer_asset.amount,
        spread_amount: offer_amount
            .saturating_sub(before_commission.to_uint128_with_precision(offer_precision)?),
        commission_amount: fee_info
            .total_fee_rate
            .checked_mul_uint128(before_commission.to_uint128_with_precision(ask_precision)?)?,
        referral_amount,
    })
}

/// Returns information about cumulative prices for the assets in the pool using a [`CumulativePricesResponse`] object.
pub fn query_cumulative_prices(deps: Deps, env: Env) -> StdResult<CumulativePricesResponse> {
    let mut config = CONFIG.load(deps.storage)?;
    let (assets, total_share) = pool_info(deps.querier, &config)?;
    let decimal_assets = assets
        .iter()
        .cloned()
        .map(|asset| {
            let precision = get_precision(deps.storage, &asset.info)?;
            asset.to_decimal_asset(precision)
        })
        .collect::<StdResult<Vec<DecimalAsset>>>()?;

    accumulate_prices(deps, &env, &mut config, &decimal_assets)
        .map_err(|err| StdError::generic_err(format!("{err}")))?;

    Ok(CumulativePricesResponse {
        assets,
        total_share,
        cumulative_prices: config.cumulative_prices,
    })
}

/// Returns the pair contract configuration in a [`ConfigResponse`] object.
pub fn query_config(deps: Deps, env: Env) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        block_time_last: config.block_time_last,
        params: Some(to_binary(&StablePoolConfig {
            amp: Decimal::from_ratio(compute_current_amp(&config, &env)?, AMP_PRECISION),
        })?),
        owner: config.owner,
    })
}

/// Returns information about cumulative prices for the assets in the pool using a [`CumulativePricesResponse`] object.
pub fn query_spot_price(
    deps: Deps,
    env: Env,
    offer: AssetInfo,
    ask: AssetInfo,
) -> Result<SpotPriceResponse, ContractError> {
    let from = offer.validate(deps.api)?;
    let to = ask.validate(deps.api)?;

    let config = CONFIG.load(deps.storage)?;
    let (assets, _) = pool_info(deps.querier, &config)?;
    let decimal_assets = assets
        .iter()
        .cloned()
        .map(|asset| {
            let precision = get_precision(deps.storage, &asset.info)?;
            asset.to_decimal_asset(precision)
        })
        .collect::<StdResult<Vec<DecimalAsset>>>()?;

    let price = calc_spot_price(deps, &env, &config, &from, &to, &decimal_assets)?;
    Ok(SpotPriceResponse { price })
}

/// Returns information about cumulative prices for the assets in the pool using a [`CumulativePricesResponse`] object.
pub fn query_spot_price_prediction(
    deps: Deps,
    env: Env,
    offer: AssetInfo,
    ask: AssetInfo,
    max_trade: Uint128,
    target_price: Decimal,
    iterations: u8,
) -> Result<SpotPricePredictionResponse, ContractError> {
    let from = offer.validate(deps.api)?;
    let to = ask.validate(deps.api)?;

    ensure!(
        max_trade > Uint128::zero(),
        ContractError::SpotPriceInvalidMaxTrade {}
    );
    ensure!(
        target_price > Decimal::zero(),
        ContractError::SpotPriceInvalidTargetPrice {}
    );
    ensure!(
        iterations > 0 && iterations <= 100,
        ContractError::SpotPriceInvalidIterations {}
    );

    let config = CONFIG.load(deps.storage)?;
    let (assets, _) = pool_info(deps.querier, &config)?;
    let decimal_assets = assets
        .iter()
        .cloned()
        .map(|asset| {
            let precision = get_precision(deps.storage, &asset.info)?;
            asset.to_decimal_asset(precision)
        })
        .collect::<StdResult<Vec<DecimalAsset>>>()?;

    let trade = find_spot_price(
        deps,
        &env,
        &config,
        from,
        to,
        decimal_assets,
        max_trade,
        target_price,
        iterations,
    )?;
    Ok(SpotPricePredictionResponse { trade })
}

/// Returns the total amount of assets in the pool as well as the total amount of LP tokens currently minted.
pub fn pool_info(
    querier: QuerierWrapper,
    config: &Config,
) -> StdResult<(Vec<AssetValidated>, Uint128)> {
    let pools = config
        .pair_info
        .query_pools(&querier, &config.pair_info.contract_addr)?;
    let total_share = query_supply(&querier, &config.pair_info.liquidity_token)?;

    Ok((pools, total_share))
}

/// Updates the pool configuration with the specified parameters in the `params` variable.
///
/// * **params** new parameter values.
pub fn update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    params: Binary,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let factory_config = query_factory_config(&deps.querier, &config.factory_addr)?;

    if info.sender
        != if let Some(ref owner) = config.owner {
            owner.to_owned()
        } else {
            factory_config.owner
        }
    {
        return Err(ContractError::Unauthorized {});
    }

    match from_binary::<StablePoolUpdateParams>(&params)? {
        StablePoolUpdateParams::StartChangingAmp {
            next_amp,
            next_amp_time,
        } => start_changing_amp(config, deps, env, next_amp, next_amp_time)?,
        StablePoolUpdateParams::StopChangingAmp {} => stop_changing_amp(config, deps, env)?,
    }

    Ok(Response::default())
}

/// Start changing the AMP value.
///
/// * **next_amp** new value for AMP.
///
/// * **next_amp_time** end time when the pool amplification will be equal to `next_amp`.
fn start_changing_amp(
    mut config: Config,
    deps: DepsMut,
    env: Env,
    next_amp: u64,
    next_amp_time: u64,
) -> Result<(), ContractError> {
    if next_amp == 0 || next_amp > MAX_AMP {
        return Err(ContractError::IncorrectAmp { max_amp: MAX_AMP });
    }

    let current_amp = compute_current_amp(&config, &env)?.u64();

    let next_amp_with_precision = next_amp * AMP_PRECISION;

    if next_amp_with_precision * MAX_AMP_CHANGE < current_amp
        || next_amp_with_precision > current_amp * MAX_AMP_CHANGE
    {
        return Err(ContractError::MaxAmpChangeAssertion {
            max_amp_change: MAX_AMP_CHANGE,
        });
    }

    let block_time = env.block.time.seconds();

    if block_time < config.init_amp_time + MIN_AMP_CHANGING_TIME
        || next_amp_time < block_time + MIN_AMP_CHANGING_TIME
    {
        return Err(ContractError::MinAmpChangingTimeAssertion {
            min_amp_changing_time: MIN_AMP_CHANGING_TIME,
        });
    }

    config.init_amp = current_amp;
    config.next_amp = next_amp_with_precision;
    config.init_amp_time = block_time;
    config.next_amp_time = next_amp_time;

    CONFIG.save(deps.storage, &config)?;

    Ok(())
}

/// Stop changing the AMP value.
fn stop_changing_amp(mut config: Config, deps: DepsMut, env: Env) -> StdResult<()> {
    let current_amp = compute_current_amp(&config, &env)?;
    let block_time = env.block.time.seconds();

    config.init_amp = current_amp.u64();
    config.next_amp = current_amp.u64();
    config.init_amp_time = block_time;
    config.next_amp_time = block_time;

    // now (block_time < next_amp_time) is always False, so we return the saved AMP
    CONFIG.save(deps.storage, &config)?;

    Ok(())
}

/// Compute the current pool D value.
fn query_compute_d(deps: Deps, env: Env) -> StdResult<Uint128> {
    let config = CONFIG.load(deps.storage)?;

    let amp = compute_current_amp(&config, &env)?;
    let pools = config
        .pair_info
        .query_pools_decimal(&deps.querier, env.contract.address)?
        .into_iter()
        .map(|pool| pool.amount)
        .collect::<Vec<_>>();

    compute_d(amp, &pools, config.greatest_precision)
        .map_err(|_| StdError::generic_err("Failed to calculate the D"))?
        .to_uint128_with_precision(config.greatest_precision)
}

/// Updates the config's target rate from the configured lsd hub contract if it is outdated.
/// Returns `true` if the target rate was updated, `false` otherwise.
fn update_target_rate(
    querier: QuerierWrapper<Empty>,
    config: &mut Config,
    env: &Env,
) -> StdResult<bool> {
    if let Some(lsd) = &mut config.lsd {
        let now = env.block.time.seconds();
        if now < lsd.last_target_query + lsd.target_rate_epoch {
            // target rate is up to date
            return Ok(false);
        }

        let response: TargetValueResponse =
            querier.query_wasm_smart(&lsd.lsd_hub, &TargetQuery::TargetValue {})?;

        lsd.target_rate = response.target_value;
        lsd.last_target_query = now;

        Ok(true)
    } else {
        Ok(false)
    }
}
