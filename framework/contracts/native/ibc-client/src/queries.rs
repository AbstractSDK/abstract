use abstract_core::{
    ibc_client::{
        state::{Config, ACCOUNTS, ADMIN, CONFIG, REMOTE_HOST, REMOTE_PROXY},
        AccountResponse, ConfigResponse, HostResponse, ListAccountsResponse,
        ListRemoteHostsResponse, ListRemoteProxysResponse,
    },
    objects::{chain_name::ChainName, AccountId},
};
use cosmwasm_std::{Deps, Env, Order, StdError, StdResult};
use cw_storage_plus::Bound;

pub fn list_accounts(
    deps: Deps,
    start: Option<(AccountId, ChainName)>,
    limit: Option<u32>,
) -> StdResult<ListAccountsResponse> {
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

pub fn list_remote_hosts(deps: Deps) -> StdResult<ListRemoteHostsResponse> {
    let hosts = REMOTE_HOST
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<_>>()?;
    Ok(ListRemoteHostsResponse { hosts })
}

pub fn list_remote_proxys(deps: Deps) -> StdResult<ListRemoteProxysResponse> {
    let proxys = REMOTE_PROXY
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<_>>()?;
    Ok(ListRemoteProxysResponse { proxys })
}

pub fn config(deps: Deps, env: Env) -> StdResult<ConfigResponse> {
    let chain = ChainName::new(&env);
    let Config {
        version_control_address,
    } = CONFIG.load(deps.storage)?;
    let admin = ADMIN.get(deps)?.unwrap();
    Ok(ConfigResponse {
        admin: admin.into(),
        chain: chain.into_string(),
        version_control_address: version_control_address.into_string(),
    })
}

/// Returns the remote-host and polytone proxy addresses
pub fn host(deps: Deps, host_chain: ChainName) -> StdResult<HostResponse> {
    let remote_host = REMOTE_HOST.may_load(deps.storage, &host_chain)?;
    let remote_polytone_proxy = REMOTE_PROXY.may_load(deps.storage, &host_chain)?;
    Ok(HostResponse {
        remote_host,
        remote_polytone_proxy,
    })
}

pub fn account(
    deps: Deps,
    host_chain: String,
    account_id: AccountId,
) -> StdResult<AccountResponse> {
    let host_chain = ChainName::from(host_chain);
    host_chain.check().unwrap();
    let remote_proxy_addr = ACCOUNTS.load(deps.storage, (&account_id, &host_chain))?;
    Ok(AccountResponse { remote_proxy_addr })
}
