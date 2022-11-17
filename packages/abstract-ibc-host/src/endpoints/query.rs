use abstract_sdk::base::{Handler, QueryEndpoint};
use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdError, StdResult};

use abstract_sdk::os::ibc_host::{
    AccountInfo, AccountResponse, BaseQueryMsg, HostConfigResponse, ListAccountsResponse, QueryMsg,
};

use crate::{
    state::{Host, ACCOUNTS},
    HostError,
};

/// Where we dispatch the queries for the Host
/// These ExtensionQueryMsg declarations can be found in `abstract_sdk::os::common_module::app_msg`
impl<
        Error: From<cosmwasm_std::StdError> + From<HostError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > QueryEndpoint
    for Host<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
{
    type QueryMsg = QueryMsg<Self::CustomQueryMsg>;
    fn query(&self, deps: Deps, env: Env, msg: Self::QueryMsg) -> Result<Binary, StdError> {
        match msg {
            QueryMsg::App(extension_query) => {
                self.query_handler()?(deps, env, self, extension_query)
            }
            QueryMsg::Base(base_query) => {
                self.base_query(deps, env, base_query).map_err(From::from)
            }
        }
    }
}
impl<
        Error: From<cosmwasm_std::StdError> + From<HostError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > Host<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
{
    fn base_query(&self, deps: Deps, _env: Env, query: BaseQueryMsg) -> StdResult<Binary> {
        match query {
            BaseQueryMsg::Config {} => to_binary(&self.dapp_config(deps)?),
            BaseQueryMsg::Account {
                client_chain,
                os_id,
            } => to_binary(&query_account(deps, client_chain, os_id)?),
            BaseQueryMsg::ListAccounts {} => to_binary(&query_list_accounts(deps)?),
        }
    }
    fn dapp_config(&self, deps: Deps) -> StdResult<HostConfigResponse> {
        let state = self.base_state.load(deps.storage)?;
        Ok(HostConfigResponse {
            ans_host_address: state.ans_host.address,
        })
    }
}
pub fn query_account(deps: Deps, channel_id: String, os_id: u32) -> StdResult<AccountResponse> {
    let account = ACCOUNTS.may_load(deps.storage, (&channel_id, os_id))?;
    Ok(AccountResponse {
        account: account.map(Into::into),
    })
}

pub fn query_list_accounts(deps: Deps) -> StdResult<ListAccountsResponse> {
    let accounts = ACCOUNTS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let ((channel_id, os_id), account) = item?;
            Ok(AccountInfo {
                account: account.into(),
                channel_id,
                os_id,
            })
        })
        .collect::<StdResult<_>>()?;
    Ok(ListAccountsResponse { accounts })
}
