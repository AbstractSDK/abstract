use crate::state::{ContractError, Host, ACCOUNTS};
use abstract_core::objects::AccountId;
use abstract_sdk::{
    base::{Handler, QueryEndpoint},
    core::ibc_host::{
        AccountInfo, AccountResponse, BaseQueryMsg, HostConfigResponse, ListAccountsResponse,
        QueryMsg,
    },
};
use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdResult};

/// Where we dispatch the queries for the Host
/// These ApiQueryMsg declarations can be found in `abstract_sdk::core::common_module::app_msg`
impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    > QueryEndpoint
    for Host<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
{
    type QueryMsg = QueryMsg<Self::CustomQueryMsg>;
    fn query(&self, deps: Deps, env: Env, msg: Self::QueryMsg) -> Result<Binary, Error> {
        match msg {
            QueryMsg::Module(api_query) => self.query_handler()?(deps, env, self, api_query),
            QueryMsg::Base(base_query) => {
                self.base_query(deps, env, base_query).map_err(From::from)
            }
        }
    }
}
impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
    Host<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg, SudoMsg>
{
    fn base_query(&self, deps: Deps, _env: Env, query: BaseQueryMsg) -> StdResult<Binary> {
        match query {
            BaseQueryMsg::Config {} => to_binary(&self.dapp_config(deps)?),
            BaseQueryMsg::Account {
                client_chain,
                account_id,
            } => to_binary(&query_account(deps, client_chain, account_id)?),
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

pub fn query_account(
    deps: Deps,
    channel_id: String,
    account_id: AccountId,
) -> StdResult<AccountResponse> {
    let account = ACCOUNTS.may_load(deps.storage, (&channel_id, account_id))?;
    Ok(AccountResponse {
        account: account.map(Into::into),
    })
}

pub fn query_list_accounts(deps: Deps) -> StdResult<ListAccountsResponse> {
    let accounts = ACCOUNTS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let ((channel_id, account_id), account) = item?;
            Ok(AccountInfo {
                account: account.into(),
                channel_id,
                account_id,
            })
        })
        .collect::<StdResult<_>>()?;
    Ok(ListAccountsResponse { accounts })
}
