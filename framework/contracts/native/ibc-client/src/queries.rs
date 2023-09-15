use abstract_core::{
    ibc_client::{
        state::{Config, ACCOUNTS, ADMIN, CHANNELS, CONFIG, LATEST_QUERIES},
        AccountInfo, AccountResponse, ConfigResponse, LatestQueryResponse, ListAccountsResponse,
        ListChannelsResponse,
    },
    objects::AccountId,
};
use cosmwasm_std::{Deps, Order, StdResult};

pub fn query_latest_ibc_query_result(
    deps: Deps,
    host_chain: String,
    account_id: AccountId,
) -> StdResult<LatestQueryResponse> {
    let channel = CHANNELS.load(deps.storage, &host_chain)?;
    LATEST_QUERIES.load(deps.storage, (&channel, account_id))
}

// TODO: paging
pub fn query_list_accounts(deps: Deps) -> StdResult<ListAccountsResponse> {
    let accounts = ACCOUNTS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|r| {
            let ((channel_id, account_id), account) = r?;
            Ok(AccountInfo::convert(channel_id, account_id, account))
        })
        .collect::<StdResult<_>>()?;
    Ok(ListAccountsResponse { accounts })
}

pub fn query_list_channels(deps: Deps) -> StdResult<ListChannelsResponse> {
    let channels = CHANNELS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<_>>()?;
    Ok(ListChannelsResponse { channels })
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let Config {
        chain,
        version_control_address,
    } = CONFIG.load(deps.storage)?;
    let admin = ADMIN.get(deps)?.unwrap();
    Ok(ConfigResponse {
        admin: admin.into(),
        chain,
        version_control_address: version_control_address.into_string(),
    })
}

pub fn query_account(
    deps: Deps,
    host_chain: String,
    account_id: AccountId,
) -> StdResult<AccountResponse> {
    let channel = CHANNELS.load(deps.storage, &host_chain)?;
    let account = ACCOUNTS.load(deps.storage, (&channel, account_id))?;
    Ok(account.into())
}
