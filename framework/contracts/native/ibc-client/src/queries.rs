use std::str::FromStr;

use abstract_core::{
    ibc_client::{
        state::{Config, ACCOUNTS, ADMIN, CONFIG, IBC_INFRA},
        AccountResponse, ConfigResponse, HostResponse, ListAccountsResponse,
        ListIbcInfrastructureResponse, ListRemoteHostsResponse, ListRemoteProxiesResponse,
    },
    objects::{chain_name::ChainName, AccountId},
    AbstractError,
};
use cosmwasm_std::{Deps, Order, StdError, StdResult};
use cw_storage_plus::Bound;

use crate::contract::IbcClientResult;

pub fn list_accounts(
    deps: Deps,
    start: Option<(AccountId, String)>,
    limit: Option<u32>,
) -> IbcClientResult<ListAccountsResponse> {
    let start = start
        .map(|s| {
            let chain = ChainName::from_str(&s.1)?;
            Ok::<_, AbstractError>((s.0, chain))
        })
        .transpose()?;

    let accounts: Vec<(
        AccountId,
        abstract_core::objects::chain_name::ChainName,
        String,
    )> = cw_paginate::paginate_map(
        &ACCOUNTS,
        deps.storage,
        start.as_ref().map(|s| Bound::exclusive((&s.0, &s.1))),
        limit,
        |(a, c), s| Ok::<_, StdError>((a, c, s)),
    )?;

    Ok(ListAccountsResponse { accounts })
}

// No need for pagination here, not a lot of chains
pub fn list_remote_hosts(deps: Deps) -> IbcClientResult<ListRemoteHostsResponse> {
    let hosts = IBC_INFRA
        .range(deps.storage, None, None, Order::Ascending)
        .map(|c| c.map(|(chain, counterpart)| (chain, counterpart.remote_abstract_host)))
        .collect::<StdResult<_>>()?;
    Ok(ListRemoteHostsResponse { hosts })
}

// No need for pagination here, not a lot of chains
pub fn list_remote_proxies(deps: Deps) -> IbcClientResult<ListRemoteProxiesResponse> {
    let proxies = IBC_INFRA
        .range(deps.storage, None, None, Order::Ascending)
        .map(|c| c.map(|(chain, counterpart)| (chain, counterpart.remote_proxy)))
        .collect::<StdResult<_>>()?;
    Ok(ListRemoteProxiesResponse { proxies })
}

// No need for pagination here, not a lot of chains
pub fn list_ibc_counterparts(deps: Deps) -> IbcClientResult<ListIbcInfrastructureResponse> {
    let counterparts = IBC_INFRA
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<_>>()?;
    Ok(ListIbcInfrastructureResponse { counterparts })
}

pub fn config(deps: Deps) -> IbcClientResult<ConfigResponse> {
    let Config {
        version_control,
        ans_host,
    } = CONFIG.load(deps.storage)?;
    let admin = ADMIN.get(deps)?.unwrap();
    Ok(ConfigResponse {
        admin: admin.into(),
        ans_host: ans_host.address.to_string(),
        version_control_address: version_control.address.into_string(),
    })
}

/// Returns the remote-host and polytone proxy addresses (useful for registering the proxy on the host)
pub fn host(deps: Deps, host_chain: String) -> IbcClientResult<HostResponse> {
    let host_chain = ChainName::from_str(&host_chain)?;
    let ibc_counterpart = IBC_INFRA.load(deps.storage, &host_chain)?;
    let remote_host = ibc_counterpart.remote_abstract_host;
    let remote_polytone_proxy = ibc_counterpart.remote_proxy;
    Ok(HostResponse {
        remote_host,
        remote_polytone_proxy,
    })
}

pub fn account(
    deps: Deps,
    host_chain: String,
    account_id: AccountId,
) -> IbcClientResult<AccountResponse> {
    let host_chain = ChainName::from_str(&host_chain)?;
    let remote_proxy_addr = ACCOUNTS.load(deps.storage, (&account_id, &host_chain))?;
    Ok(AccountResponse { remote_proxy_addr })
}
