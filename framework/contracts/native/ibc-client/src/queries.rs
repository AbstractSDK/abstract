use abstract_core::{
    ibc_client::{
        state::{Config, ACCOUNTS, ADMIN, CONFIG, REMOTE_HOST},
        AccountResponse, ConfigResponse, ListAccountsResponse, ListRemoteHostsResponse,
    },
    objects::{chain_name::ChainName, AccountId},
};
use cosmwasm_std::{Deps, Env, Order, StdResult};

// TODO: paging
pub fn list_accounts(deps: Deps) -> StdResult<ListAccountsResponse> {
    let accounts: StdResult<
        Vec<(
            AccountId,
            abstract_core::objects::chain_name::ChainName,
            String,
        )>,
    > = ACCOUNTS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|r| r.map(|((a, c), s)| (a, c, s)))
        .collect();

    Ok(ListAccountsResponse {
        accounts: accounts?,
    })
}

pub fn list_remote_hosts(deps: Deps) -> StdResult<ListRemoteHostsResponse> {
    let hosts = REMOTE_HOST
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<_>>()?;
    Ok(ListRemoteHostsResponse { hosts })
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
