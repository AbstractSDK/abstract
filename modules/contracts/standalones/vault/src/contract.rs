use std::collections::HashMap;

use abstract_oracle_adapter::OracleInterface;
use abstract_standalone::sdk::AbstractResponse;
use cosmwasm_std::{
    ensure, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, StdResult,
};
use cw_vault_standard::VaultStandardInfoResponse;

use crate::{
    msg::{
        MyStandaloneExecuteMsg, MyStandaloneInstantiateMsg, MyStandaloneMigrateMsg,
        MyStandaloneQueryMsg,
    },
    state::{Config, CONFIG},
    MyStandalone, MyStandaloneError, MyStandaloneResult, MY_STANDALONE,
};

const TODO_REPLY_ID: u64 = 0;
const ORACLE_NAME: &str = {
    #[cfg(feature = "pyth")]
    {
        abstract_oracle_adapter::oracles::PYTH
    }
};

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: MyStandaloneInstantiateMsg,
) -> MyStandaloneResult {
    // Init standalone as module
    let is_migratable = true;
    MY_STANDALONE.instantiate(deps.branch(), info, msg.base, is_migratable)?;

    let config: Config = Config {
        denom_whitelist: msg.denom_whitelist,
        max_age: msg.max_age,
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(MY_STANDALONE
        .response("init")
        .add_message(crate::tokenfactory::create_denom(&env)))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: MyStandaloneExecuteMsg,
) -> MyStandaloneResult {
    let standalone = MY_STANDALONE;
    match msg {
        MyStandaloneExecuteMsg::Deposit { recipient, .. } => {
            deposit(deps, env, info, standalone, recipient)
        }
        MyStandaloneExecuteMsg::Redeem { recipient, .. } => todo!(),
        MyStandaloneExecuteMsg::VaultExtension(_) => todo!(),
    }
}

fn deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module: MyStandalone,
    recipient: Option<String>,
) -> MyStandaloneResult {
    let config = CONFIG.load(deps.storage)?;

    let recipient_addr = recipient
        .map(|human| deps.api.addr_validate(&human))
        .transpose()?
        .unwrap_or(info.sender.clone());

    // Check every asset is whitelisted
    // TODO: move price calculation somewhere else, we don't always rely on the oracle
    // let mut price_map = HashMap::new();
    for coin in info.funds.iter() {
        ensure!(
            config.denom_whitelist.contains(&coin.denom),
            MyStandaloneError::NotWhitelistedAsset(coin.denom.clone())
        );
        //     let denom = coin.denom.clone();
        //     let price_source_key = config
        //         .price_sources
        //         .get(&denom)
        //         .ok_or(MyStandaloneError::NotWhitelistedAsset(denom.clone()))?
        //         .to_owned();
        //     let price_response = module
        //         .oracle(deps.as_ref(), ORACLE_NAME.to_owned())
        //         .price(price_source_key, config.max_age)?;

        //     price_map.insert(denom, price_response.price);
    }
    Ok(module.response("deposit"))
}

// #[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: MyStandaloneQueryMsg) -> StdResult<Binary> {
    let standalone = &MY_STANDALONE;
    match msg {
        MyStandaloneQueryMsg::VaultStandardInfo {} => to_json_binary(&VaultStandardInfoResponse {
            version: cw_vault_standard::VERSION.to_owned(),
            extensions: vec![],
        }),
        MyStandaloneQueryMsg::Info {} => todo!(),
        MyStandaloneQueryMsg::PreviewDeposit { amount } => todo!(),
        MyStandaloneQueryMsg::PreviewRedeem { amount } => todo!(),
        MyStandaloneQueryMsg::TotalAssets {} => todo!(),
        MyStandaloneQueryMsg::TotalVaultTokenSupply {} => todo!(),
        MyStandaloneQueryMsg::VaultTokenExchangeRate { quote_denom } => todo!(),
        MyStandaloneQueryMsg::ConvertToShares { amount } => todo!(),
        MyStandaloneQueryMsg::ConvertToAssets { amount } => todo!(),
        MyStandaloneQueryMsg::VaultExtension(_) => unreachable!("No extensions enabled"),
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> MyStandaloneResult {
    match msg.id {
        self::TODO_REPLY_ID => Ok(crate::MY_STANDALONE.response("TODO:")),
        _ => todo!(),
    }
}

/// Handle the standalone migrate msg
#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MyStandaloneMigrateMsg) -> MyStandaloneResult {
    // The Abstract Standalone object does version checking and
    MY_STANDALONE.migrate(deps)?;
    Ok(MY_STANDALONE.response("migrate"))
}
