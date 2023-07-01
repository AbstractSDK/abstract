#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, ensure_eq, to_binary, Addr, Binary, Coin, Deps, DepsMut, Empty, Env, MessageInfo, Order,
    Reply, Response, StdResult, SubMsg, Uint128, WasmMsg,
};

use cw2::{get_contract_version, set_contract_version};
use cw_utils::ensure_from_older_version;
use wasmswap::msg::InfoResponse;
use wyndex::asset::{Asset, AssetInfo};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, MigrateMsg, QueryMsg};
use crate::state::{MigrateConfig, MigrateStakersConfig, DESTINATION, MIGRATION};

// this is the contract we are migrating from
pub const STAKE_CW20_NAME: &str = "crates.io:stake_cw20";

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:junoswap-staking";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> Result<Response, ContractError> {
    Err(ContractError::NotImplemented)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::MigrationFinished {} => {
            let no_stakers = stake_cw20::state::STAKED_BALANCES
                .keys(_deps.storage, None, None, Order::Ascending)
                .next()
                .is_none();
            Ok(to_binary(&no_stakers)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    // if we got an init, then do from junoswap loop
    if let Some(msg) = msg.init {
        // ensure contract being migrated is actually junoswap staking
        let old = get_contract_version(deps.storage)?;
        if old.contract != STAKE_CW20_NAME {
            return Err(ContractError::CannotMigrate(old.contract));
        }
        // question: check version??

        // update the cw2 contract version
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        // check the unbonding period is valid
        let factory = deps.api.addr_validate(&msg.factory)?;
        let unbonding_period = msg.unbonding_period;
        let wyndex_factory::state::Config {
            default_stake_config,
            ..
        } = wyndex_factory::state::CONFIG.query(&deps.querier, factory.clone())?;
        if !default_stake_config
            .unbonding_periods
            .iter()
            .any(|x| x == &unbonding_period)
        {
            return Err(ContractError::InvalidUnbondingPeriod(unbonding_period));
        }

        // Validate arguments and set config for future calls
        let wynddex_pool = msg
            .wynddex_pool
            .map(|p| deps.api.addr_validate(&p))
            .transpose()?;
        let config = MigrateConfig {
            migrator: deps.api.addr_validate(&msg.migrator)?,
            junoswap_pool: deps.api.addr_validate(&msg.junoswap_pool)?,
            factory,
            unbonding_period,
            wynddex_pool,
            migrate_stakers_config: None,
        };
        MIGRATION.save(deps.storage, &config)?;
    } else {
        // self-migrate
        // this is only used as a bug-fix-patch on the older version of this code.
        ensure_from_older_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        // update the cw2 contract version
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    }

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::MigrateTokens { wynddex_pool } => migrate_tokens(deps, env, info, wynddex_pool),
        ExecuteMsg::MigrateStakers { limit } => migrate_stakers(deps, env, info, limit),
    }
}

/// Allow `migrator` to pull out LP positions and send them to wynd dex pool
/// First step figures out how many LPs we have and withdraws them.
/// Follow up via reply.
pub fn migrate_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    wynddex_pool: String,
) -> Result<Response, ContractError> {
    // make sure called by proper account
    let mut migration = MIGRATION.load(deps.storage)?;
    if info.sender != migration.migrator {
        return Err(ContractError::Unauthorized);
    }

    // ensure the requested target pool is valid
    let w_pool = deps.api.addr_validate(&wynddex_pool)?;
    if let Some(ref target) = migration.wynddex_pool {
        if target != w_pool {
            return Err(ContractError::InvalidDestination(wynddex_pool));
        }
    }
    let ci = deps.querier.query_wasm_contract_info(&w_pool)?;
    if ci.creator != migration.factory {
        return Err(ContractError::InvalidDestination(wynddex_pool));
    }

    // save target pool for later reply block
    DESTINATION.save(deps.storage, &w_pool)?;

    // calculate LP tokens owner by staking contract,
    // for withdrawal and for future distribution
    let stake_cfg = stake_cw20::state::CONFIG.load(deps.storage)?;
    let token = cw20::Cw20Contract(stake_cfg.token_address);
    let balance = token.balance(&deps.querier, env.contract.address)?;

    // fill in most of the migration data now (minus wynd dex LP)
    let wyndex::pair::PairInfo {
        liquidity_token,
        staking_addr,
        ..
    } = deps
        .querier
        .query_wasm_smart(&w_pool, &wyndex::pair::QueryMsg::Pair {})?;

    // total_staked is same a balance of junoswap lp token held by this contract
    migration.migrate_stakers_config = Some(MigrateStakersConfig {
        lp_token: liquidity_token,
        staking_addr,
        total_lp_tokens: Uint128::zero(),
        total_staked: balance,
    });
    MIGRATION.save(deps.storage, &migration)?;

    // trigger withdrawal of LP tokens
    // we need to assign a cw20 allowance to let the pool burn LP
    let allowance = WasmMsg::Execute {
        contract_addr: token.0.to_string(),
        funds: vec![],
        msg: to_binary(&cw20::Cw20ExecuteMsg::IncreaseAllowance {
            spender: migration.junoswap_pool.to_string(),
            amount: balance,
            expires: None,
        })?,
    };

    // then craft the LP withdrawal message
    let withdraw = WasmMsg::Execute {
        contract_addr: migration.junoswap_pool.into_string(),
        funds: vec![],
        msg: to_binary(&wasmswap::msg::ExecuteMsg::RemoveLiquidity {
            amount: balance,
            min_token1: Uint128::zero(),
            min_token2: Uint128::zero(),
            expiration: None,
        })?,
    };

    // execute these and handle the next step in reply
    let res = Response::new()
        .add_message(allowance)
        .add_submessage(SubMsg::reply_on_success(withdraw, REPLY_ONE));
    Ok(res)
}

pub fn migrate_stakers(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    limit: u32,
) -> Result<Response, ContractError> {
    // make sure called by proper account
    let migration = MIGRATION.load(deps.storage)?;
    ensure_eq!(info.sender, migration.migrator, ContractError::Unauthorized);

    let config = migration
        .migrate_stakers_config
        .ok_or(ContractError::TokensNotMigrated)?;

    // calculate next `limit` stakers and their shares
    let stakers = find_stakers(deps.as_ref(), limit)?;

    // remove the processed stakers from the state
    remove_stakers(deps.branch(), &env, stakers.iter().map(|(addr, _)| addr))?;

    let staker_lps: Vec<_> = stakers
        .into_iter()
        .map(|(addr, stake)| {
            (
                addr.to_string(),
                stake * config.total_lp_tokens / config.total_staked,
            )
        })
        .filter(|(_, x)| !x.is_zero())
        .collect();

    // the amount of LP tokens we are migrating in this message
    let batch_lp: Uint128 = staker_lps.iter().map(|(_, x)| x).sum();

    // bonding has full info on who receives the delegation
    let bond_msg = wyndex::stake::ReceiveMsg::MassDelegate {
        unbonding_period: migration.unbonding_period,
        delegate_to: staker_lps,
    };

    // stake it all
    let stake_msg = WasmMsg::Execute {
        contract_addr: config.lp_token.to_string(),
        funds: vec![],
        msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
            contract: config.staking_addr.into_string(),
            amount: batch_lp,
            msg: to_binary(&bond_msg)?,
        })?,
    };

    Ok(Response::new().add_message(stake_msg))
}

const REPLY_ONE: u64 = 111;
const REPLY_TWO: u64 = 222;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.result.is_err() {
        return Err(ContractError::ErrorReply);
    }
    match msg.id {
        REPLY_ONE => reply_one(deps, env),
        REPLY_TWO => reply_two(deps, env),
        x => Err(ContractError::UnknownReply(x)),
    }
}

/// In this step, we deposit the new raw tokens (eg. JUNO-ATOM) into WYND DEX
/// And get some liquid WYND DEX LP tokens
pub fn reply_one(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let migration = MIGRATION.load(deps.storage)?;
    let destination = DESTINATION.load(deps.storage)?;

    // get the JS asset types and convert to WYND DEX types
    let info: InfoResponse = deps
        .querier
        .query_wasm_smart(&migration.junoswap_pool, &wasmswap::msg::QueryMsg::Info {})?;
    let assets = to_wyndex_assets(deps.as_ref(), env.contract.address, info)?;

    // figure out how to transfer these... previous cw20 allowances or
    // sending native funds inline with providing liquidity
    let (allowances, funds) = prepare_denom_deposits(&destination, &assets)?;
    let deposit = WasmMsg::Execute {
        contract_addr: destination.into_string(),
        funds,
        msg: to_binary(&wyndex::pair::ExecuteMsg::ProvideLiquidity {
            assets,
            // TODO: set some value here?
            slippage_tolerance: None,
            receiver: None,
        })?,
    };

    // add any cw20 allowances, then call to deposit the tokens and get LP
    let res = Response::new()
        .add_messages(allowances)
        .add_submessage(SubMsg::reply_on_success(deposit, REPLY_TWO));
    Ok(res)
}

fn prepare_denom_deposits(
    destination: &Addr,
    assets: &[Asset],
) -> Result<(Vec<WasmMsg>, Vec<Coin>), ContractError> {
    let mut msgs = vec![];
    let mut funds = vec![];
    prepare_denom_deposit(destination, &assets[0], &mut msgs, &mut funds)?;
    prepare_denom_deposit(destination, &assets[1], &mut msgs, &mut funds)?;
    // sort denoms for deposit
    funds.sort_by(|coin1, coin2| coin1.denom.cmp(&coin2.denom));
    Ok((msgs, funds))
}

fn prepare_denom_deposit(
    destination: &Addr,
    asset: &Asset,
    msgs: &mut Vec<WasmMsg>,
    funds: &mut Vec<Coin>,
) -> Result<(), ContractError> {
    // build allowance msg or funds to transfer for this asset
    match &asset.info {
        AssetInfo::Token(token) => {
            let embed = cw20::Cw20ExecuteMsg::IncreaseAllowance {
                spender: destination.to_string(),
                amount: asset.amount,
                expires: None,
            };
            let msg = WasmMsg::Execute {
                contract_addr: token.to_string(),
                msg: to_binary(&embed)?,
                funds: vec![],
            };
            msgs.push(msg);
        }
        AssetInfo::Native(denom) => {
            let coin = coin(asset.amount.u128(), denom);
            funds.push(coin);
        }
    }
    Ok(())
}

fn to_wyndex_assets(deps: Deps, me: Addr, info: InfoResponse) -> Result<Vec<Asset>, ContractError> {
    let asset1 = to_wyndex_asset(deps, &me, info.token1_denom)?;
    let asset2 = to_wyndex_asset(deps, &me, info.token2_denom)?;
    Ok(vec![asset1, asset2])
}

fn to_wyndex_asset(
    deps: Deps,
    me: &Addr,
    token: wasmswap_cw20::Denom,
) -> Result<Asset, ContractError> {
    let asset = match token {
        wasmswap_cw20::Denom::Native(denom) => {
            let balance = deps.querier.query_balance(me, denom)?;
            Asset {
                info: AssetInfo::Native(balance.denom),
                amount: balance.amount,
            }
        }
        wasmswap_cw20::Denom::Cw20(addr) => {
            let token = cw20::Cw20Contract(addr);
            let amount = token.balance(&deps.querier, me)?;
            Asset {
                info: AssetInfo::Token(token.0.into_string()),
                amount,
            }
        }
    };
    Ok(asset)
}

/// Finally, with those WYND DEX LP tokens, we will take them all on behalf
/// of the original JunoSwap LP stakers.
pub fn reply_two(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    // load config for LP token and staking contract
    let mut migration = MIGRATION.load(deps.storage)?;
    let config = migration.migrate_stakers_config.as_mut().unwrap();

    // how many LP do we have total
    let lp_token = cw20::Cw20Contract(config.lp_token.clone());
    let total_lp_tokens = lp_token.balance(&deps.querier, &env.contract.address)?;

    // store this for `migrate_stakers` to use
    config.total_lp_tokens = total_lp_tokens;
    MIGRATION.save(deps.storage, &migration)?;

    Ok(Response::new())
}

// query logic taken from https://github.com/cosmorama/wyndex-priv/pull/109
fn find_stakers(deps: Deps, limit: impl Into<Option<u32>>) -> StdResult<Vec<(Addr, Uint128)>> {
    let balances = stake_cw20::state::STAKED_BALANCES
        .range(deps.storage, None, None, Order::Ascending)
        .map(|stake| {
            let (addr, amount) = stake?;

            // query all pending claims and bond them as well
            let claims = stake_cw20::state::CLAIMS.query_claims(deps, &addr)?;
            let claims_sum = claims.claims.iter().map(|c| c.amount).sum::<Uint128>();

            Ok((addr, amount + claims_sum))
        });
    match limit.into() {
        Some(limit) => balances.take(limit as usize).collect(),
        None => balances.collect(),
    }
}

fn remove_stakers<'a>(
    deps: DepsMut,
    env: &Env,
    stakers: impl Iterator<Item = &'a Addr>,
) -> Result<(), ContractError> {
    for staker in stakers {
        stake_cw20::state::STAKED_BALANCES.remove(deps.storage, staker, env.block.height)?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::coin;

    #[test]
    fn prepare_denom_deposits_sorted() {
        let destination = Addr::unchecked("destination");
        let assets = vec![
            Asset {
                info: AssetInfo::Native("uusdc".to_owned()),
                amount: Uint128::new(9_000_000_000u128),
            },
            Asset {
                info: AssetInfo::Native("ujuno".to_owned()),
                amount: Uint128::new(5_000_000_000u128),
            },
        ];

        let (_, rcoins) = prepare_denom_deposits(&destination, &assets).unwrap();
        assert_eq!(
            rcoins,
            vec![
                coin(5_000_000_000u128, "ujuno".to_owned()),
                coin(9_000_000_000u128, "uusdc".to_owned())
            ]
        );

        let assets = vec![
            Asset {
                info: AssetInfo::Native("uusdc".to_owned()),
                amount: Uint128::new(9_000_000_000u128),
            },
            Asset {
                info: AssetInfo::Native(
                    "ibc/0C1FFD27A01B116F10F0BC624A6A24190BC9C57B27837E23E3B43C34A193967C"
                        .to_owned(),
                ),
                amount: Uint128::new(1_000_000_000u128),
            },
        ];
        let (_, rcoins) = prepare_denom_deposits(&destination, &assets).unwrap();
        assert_eq!(
            rcoins,
            vec![
                coin(
                    1_000_000_000u128,
                    "ibc/0C1FFD27A01B116F10F0BC624A6A24190BC9C57B27837E23E3B43C34A193967C"
                        .to_owned()
                ),
                coin(9_000_000_000u128, "uusdc".to_owned())
            ]
        );

        let assets = vec![
            Asset {
                info: AssetInfo::Token("juno1wynd".to_owned()),
                amount: Uint128::new(69_000_000_000u128),
            },
            Asset {
                info: AssetInfo::Native(
                    "ibc/0C1FFD27A01B116F10F0BC624A6A24190BC9C57B27837E23E3B43C34A193967C"
                        .to_owned(),
                ),
                amount: Uint128::new(1_000_000_000u128),
            },
        ];
        let (_, rcoins) = prepare_denom_deposits(&destination, &assets).unwrap();
        assert_eq!(
            rcoins,
            vec![coin(
                1_000_000_000u128,
                "ibc/0C1FFD27A01B116F10F0BC624A6A24190BC9C57B27837E23E3B43C34A193967C".to_owned()
            ),]
        );
    }
}
