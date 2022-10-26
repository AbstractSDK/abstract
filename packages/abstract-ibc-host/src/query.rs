use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdResult};

use abstract_os::{
    ibc_host::{AccountInfo, AccountResponse, HostConfigResponse, ListAccountsResponse},
    ibc_host::{BaseQueryMsg, QueryMsg},
};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    state::{Host, ACCOUNTS},
    HostError,
};

pub type HostQueryHandlerFn<Q, QueryError> = Option<fn(Deps, Env, Q) -> Result<Binary, QueryError>>;

/// Where we dispatch the queries for the Host
/// These ApiQueryMsg declarations can be found in `abstract_os::common_module::add_on_msg`
impl<'a, T: Serialize + DeserializeOwned> Host<'a, T> {
    pub fn handle_query<
        Q: Serialize + DeserializeOwned,
        QueryError: From<cosmwasm_std::StdError> + From<HostError>,
    >(
        &self,
        deps: Deps,
        env: Env,
        msg: QueryMsg<Q>,
        custom_query_handler: HostQueryHandlerFn<Q, QueryError>,
    ) -> Result<Binary, QueryError> {
        match msg {
            QueryMsg::App(api_query) => custom_query_handler
                .map(|func| func(deps, env, api_query))
                .transpose()?
                .ok_or_else(|| HostError::NoCustomQueries {}.into()),
            QueryMsg::Base(base_query) => self.query(deps, env, base_query).map_err(From::from),
        }
    }

    fn query(&self, deps: Deps, _env: Env, query: BaseQueryMsg) -> StdResult<Binary> {
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
            memory_address: state.memory.address,
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
