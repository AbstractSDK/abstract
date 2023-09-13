use std::str::FromStr;

use abstract_core::{
    ibc_host::{
        state::{CHAIN_PROXYS, CONFIG},
        ConfigResponse, RegisteredChainResponse, RegisteredChainsResponse,
    },
    objects::chain_name::ChainName,
};
use abstract_sdk::core::ibc_host::QueryMsg;
use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdResult};

use crate::contract::HostResult;

pub fn query(deps: Deps, _env: Env, query: QueryMsg) -> HostResult<Binary> {
    match query {
        QueryMsg::Config {} => to_binary(&dapp_config(deps)?),
        QueryMsg::RegisteredChains {} => to_binary(&registered_chains(deps)?),
        QueryMsg::AssociatedClient { chain } => to_binary(&associated_client(deps, chain)?),
    }
    .map_err(Into::into)
}
fn dapp_config(deps: Deps) -> HostResult<ConfigResponse> {
    let state = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        ans_host_address: state.ans_host.address,
        account_factory_address: state.account_factory,
        version_control_address: state.version_control,
    })
}

// Potentiel TODO : should we use pagination here ?
fn registered_chains(deps: Deps) -> HostResult<RegisteredChainsResponse> {
    let chains = CHAIN_PROXYS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?
        .into_iter()
        .map(|(name, proxy)| (name, proxy.to_string()))
        .collect();

    Ok(RegisteredChainsResponse { chains })
}

fn associated_client(deps: Deps, chain: String) -> HostResult<RegisteredChainResponse> {
    let proxy = CHAIN_PROXYS.load(deps.storage, &ChainName::from_str(&chain)?)?;

    Ok(RegisteredChainResponse {
        proxy: proxy.to_string(),
    })
}
