use abstract_core::{
    ibc_host::{ConfigResponse, RegisteredChainResponse, RegisteredChainsResponse, state::{CHAIN_PROXYS, CONFIG}},
    objects::chain_name::ChainName,
};
use abstract_sdk::core::ibc_host::QueryMsg;
use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdResult};


pub fn query(deps: Deps, _env: Env, query: QueryMsg) -> StdResult<Binary> {
    match query {
        QueryMsg::Config {} => to_binary(&dapp_config(deps)?),
        QueryMsg::RegisteredChains {} => to_binary(&registered_chains(deps)?),
        QueryMsg::AssociatedClient { chain } => to_binary(&associated_client(deps, chain)?),
    }
}
fn dapp_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        ans_host_address: state.ans_host.address,
        account_factory_address: state.account_factory,
        version_control_address: state.version_control,
    })
}


// Potentiel TODO : should we use pagination here ?
fn registered_chains(deps: Deps) -> StdResult<RegisteredChainsResponse> {
    let chains = CHAIN_PROXYS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?
        .into_iter()
        .map(|(name, proxy)| (name, proxy.to_string()))
        .collect();

    Ok(RegisteredChainsResponse { chains })
}

fn associated_client(deps: Deps, chain: String) -> StdResult<RegisteredChainResponse> {
    let proxy = CHAIN_PROXYS.load(deps.storage, &ChainName::from(chain))?;

    Ok(RegisteredChainResponse {
        proxy: proxy.to_string(),
    })
}
