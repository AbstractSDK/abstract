use std::str::FromStr;

use abstract_core::{
    ibc_host::{
        state::{CHAIN_PROXIES, CONFIG},
        ConfigResponse, RegisteredChainResponse, RegisteredChainsResponse,
    },
    objects::chain_name::ChainName,
};
use abstract_sdk::core::ibc_host::QueryMsg;
use cosmwasm_std::{to_binary, Binary, Deps, Env};
use cw_storage_plus::Bound;

use crate::{contract::HostResult, HostError};

pub fn query(deps: Deps, _env: Env, query: QueryMsg) -> HostResult<Binary> {
    match query {
        QueryMsg::Config {} => to_binary(&dapp_config(deps)?),
        QueryMsg::RegisteredChains { start, limit } => {
            to_binary(&registered_chains(deps, start, limit)?)
        }
        QueryMsg::AssociatedClient { chain } => to_binary(&associated_client(deps, chain)?),
    }
    .map_err(Into::into)
}
fn dapp_config(deps: Deps) -> HostResult<ConfigResponse> {
    let state = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        ans_host_address: state.ans_host.address,
        account_factory_address: state.account_factory,
        version_control_address: state.version_control.address,
    })
}

// Potentiel TODO : should we use pagination here ?
fn registered_chains(
    deps: Deps,
    start: Option<String>,
    limit: Option<u32>,
) -> HostResult<RegisteredChainsResponse> {
    let start = start.map(ChainName::from_string).transpose()?;

    let chains = cw_paginate::paginate_map(
        &CHAIN_PROXIES,
        deps.storage,
        start.as_ref().map(Bound::exclusive),
        limit,
        |name, proxy| Ok::<_, HostError>((name, proxy.to_string())),
    )?;

    Ok(RegisteredChainsResponse { chains })
}

fn associated_client(deps: Deps, chain: String) -> HostResult<RegisteredChainResponse> {
    let proxy = CHAIN_PROXIES.load(deps.storage, &ChainName::from_str(&chain)?)?;

    Ok(RegisteredChainResponse {
        proxy: proxy.to_string(),
    })
}
