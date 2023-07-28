use crate::patch::execute_module_action_response;
use abstract_core::proxy::AssetConfigResponse;
use dao_dao_core::state::PAUSED;
use crate::{error::DaoProxyError, msg::*};
use abstract_proxy::*;
use abstract_core::objects::module_version::assert_contract_upgrade;
use abstract_core::objects::oracle::Oracle;
use abstract_macros::abstract_response;
use abstract_sdk::{
    core::{
        objects::account_id::ACCOUNT_ID,
        proxy::{
            state::{State, ANS_HOST, STATE},
        },
        PROXY,
    },
    feature_objects::AnsHost,
};
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, SubMsgResult, Empty,
};

use semver::Version;

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const RESPONSE_REPLY_ID: u64 = 1111;

#[abstract_response(PROXY)]
pub struct ProxyResponse;

/// The result type for the proxy contract.
pub type DaoProxyResult<T = Response> = Result<T, DaoProxyError>;

/*
    The proxy is the bank account of the account. It owns the liquidity and acts as a proxy contract.
    Whitelisted dApps construct messages for this contract. The dApps are controlled by the Manager.
*/

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> DaoProxyResult {
    panic!("Only use this contract through a migration.");
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> DaoProxyResult {
    // No actions can be performed while the DAO is paused.
    if let Some(expiration) = PAUSED.may_load(deps.storage)? {
        if !expiration.is_expired(&env.block) {
            return Err(dao_dao_core::ContractError::Paused {}.into());
        }
    }
    match msg {
        // Abstract Proxy Messages
        ExecuteMsg::ModuleAction { msgs } => abstract_proxy::commands::execute_module_action(deps, info, msgs).map_err(Into::into),
        // This action is patched on purpose to be able to differentiate between reply ids
        ExecuteMsg::ModuleActionWithData { msg } => execute_module_action_response(deps, info, msg).map_err(Into::into),
        ExecuteMsg::IbcAction { msgs } => abstract_proxy::commands::execute_ibc_action(deps, info, msgs).map_err(Into::into),
        ExecuteMsg::SetAdmin { admin } => abstract_proxy::commands::set_admin(deps, info, &admin).map_err(Into::into),
        ExecuteMsg::AddModule { module } => abstract_proxy::commands::add_module(deps, info, module).map_err(Into::into),
        ExecuteMsg::RemoveModule { module } => abstract_proxy::commands::remove_module(deps, info, module).map_err(Into::into),
        ExecuteMsg::UpdateAssets { to_add, to_remove } => {
            abstract_proxy::commands::update_assets(deps, info, to_add, to_remove).map_err(Into::into)
        },

        // DaoDao Proxy Messages
        ExecuteMsg::ExecuteAdminMsgs { msgs } => {
            dao_dao_core::contract::execute_admin_msgs(deps.as_ref(), info.sender, msgs).map_err(Into::into)
        }
        ExecuteMsg::ExecuteProposalHook { msgs } => {
            dao_dao_core::contract::execute_proposal_hook(deps.as_ref(), info.sender, msgs).map_err(Into::into)
        }
        ExecuteMsg::Pause { duration } => dao_dao_core::contract::execute_pause(deps, env, info.sender, duration).map_err(Into::into),
        ExecuteMsg::Receive(_) => dao_dao_core::contract::execute_receive_cw20(deps, info.sender).map_err(Into::into),
        ExecuteMsg::ReceiveNft(_) => dao_dao_core::contract::execute_receive_cw721(deps, info.sender).map_err(Into::into),
        ExecuteMsg::RemoveItem { key } => dao_dao_core::contract::execute_remove_item(deps, env, info.sender, key).map_err(Into::into),
        ExecuteMsg::SetItem { key, value } => dao_dao_core::contract::execute_set_item(deps, env, info.sender, key, value).map_err(Into::into),
        ExecuteMsg::UpdateConfig { config } => {
            dao_dao_core::contract::execute_update_config(deps, env, info.sender, config).map_err(Into::into)
        }
        ExecuteMsg::UpdateCw20List { to_add, to_remove } => {
            dao_dao_core::contract::execute_update_cw20_list(deps, env, info.sender, to_add, to_remove).map_err(Into::into)
        }
        ExecuteMsg::UpdateCw721List { to_add, to_remove } => {
            dao_dao_core::contract::execute_update_cw721_list(deps, env, info.sender, to_add, to_remove).map_err(Into::into)
        }
        ExecuteMsg::UpdateVotingModule { module } => {
            dao_dao_core::contract::execute_update_voting_module(env, info.sender, module).map_err(Into::into)
        }
        ExecuteMsg::UpdateProposalModules { to_add, to_disable } => {
            dao_dao_core::contract::execute_update_proposal_modules(deps, env, info.sender, to_add, to_disable).map_err(Into::into)
        }
        ExecuteMsg::NominateAdmin { admin } => {
            dao_dao_core::contract::execute_nominate_admin(deps, env, info.sender, admin).map_err(Into::into)
        }
        ExecuteMsg::AcceptAdminNomination {} => dao_dao_core::contract::execute_accept_admin_nomination(deps, info.sender).map_err(Into::into),
        ExecuteMsg::WithdrawAdminNomination {} => {
            dao_dao_core::contract::execute_withdraw_admin_nomination(deps, info.sender).map_err(Into::into)
        }
        ExecuteMsg::UpdateSubDaos { to_add, to_remove } => {
            dao_dao_core::contract::execute_update_sub_daos_list(deps, env, info.sender, to_add, to_remove).map_err(Into::into)
        }
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> DaoProxyResult {
    // Use CW2 to set the contract version, this is needed for migrations
    ACCOUNT_ID.save(deps.storage, &msg.abstract_account_id)?;
    STATE.save(deps.storage, &State { modules: vec![] })?;
    ANS_HOST.save(
        deps.storage,
        &AnsHost {
            address: deps.api.addr_validate(&msg.ans_host_address)?,
        },
    )?;
    // Don't need to setup the admin, they already have an admin field in the dao-dao contract
    // TODO, to erase
    /*
        let admin_addr = Some(dao_dao_core::contract::query_admin(deps))?;
        ADMIN.set(deps, admin_addr)?;
    */

    let version: Version = CONTRACT_VERSION.parse().unwrap();
    assert_contract_upgrade(deps.storage, PROXY, version)?;
    cw2::set_contract_version(deps.storage, PROXY, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> DaoProxyResult<Binary> {
    match msg {
        // Abstract Proxy Messages
        QueryMsg::AbstractConfig {} => to_binary(&abstract_proxy::queries::query_config(deps)?),
        QueryMsg::TotalValue {} => to_binary(&abstract_proxy::queries::query_total_value(deps, env)?),
        QueryMsg::HoldingAmount { identifier } => {
            to_binary(&abstract_proxy::queries::query_holding_amount(deps, env, identifier)?)
        }
        QueryMsg::TokenValue { identifier } => {
            to_binary(&abstract_proxy::queries::query_token_value(deps, env, identifier)?)
        }
        QueryMsg::AssetConfig { identifier } => to_binary(&AssetConfigResponse {
            price_source: Oracle::new().asset_config(deps, &identifier)?,
        }),
        QueryMsg::AssetsConfig { start_after, limit } => {
            to_binary(&abstract_proxy::queries::query_oracle_asset_config(deps, start_after, limit)?)
        }
        QueryMsg::AssetsInfo { start_after, limit } => {
            to_binary(&abstract_proxy::queries::query_oracle_asset_info(deps, start_after, limit)?)
        }
        QueryMsg::BaseAsset {} => to_binary(&abstract_proxy::queries::query_base_asset(deps)?),

        // DaoDao Proxy Messages
        QueryMsg::Admin {} => dao_dao_core::contract::query_admin(deps),
        QueryMsg::AdminNomination {} => dao_dao_core::contract::query_admin_nomination(deps),
        QueryMsg::Config {} => dao_dao_core::contract::query_config(deps),
        QueryMsg::Cw20TokenList { start_after, limit } => dao_dao_core::contract::query_cw20_list(deps, start_after, limit),
        QueryMsg::Cw20Balances { start_after, limit } => {
            dao_dao_core::contract::query_cw20_balances(deps, env, start_after, limit)
        }
        QueryMsg::Cw721TokenList { start_after, limit } => {
            dao_dao_core::contract::query_cw721_list(deps, start_after, limit)
        }
        QueryMsg::DumpState {} => dao_dao_core::contract::query_dump_state(deps, env),
        QueryMsg::GetItem { key } => dao_dao_core::contract::query_get_item(deps, key),
        QueryMsg::Info {} => dao_dao_core::contract::query_info(deps),
        QueryMsg::ListItems { start_after, limit } => dao_dao_core::contract::query_list_items(deps, start_after, limit),
        QueryMsg::PauseInfo {} => dao_dao_core::contract::query_paused(deps, env),
        QueryMsg::ProposalModules { start_after, limit } => {
            dao_dao_core::contract::query_proposal_modules(deps, start_after, limit)
        }
        QueryMsg::ProposalModuleCount {} => dao_dao_core::contract::query_proposal_module_count(deps),
        QueryMsg::TotalPowerAtHeight { height } => dao_dao_core::contract::query_total_power_at_height(deps, height),
        QueryMsg::VotingModule {} => dao_dao_core::contract::query_voting_module(deps),
        QueryMsg::VotingPowerAtHeight { address, height } => {
            dao_dao_core::contract::query_voting_power_at_height(deps, address, height)
        }
        QueryMsg::ActiveProposalModules { start_after, limit } => {
            dao_dao_core::contract::query_active_proposal_modules(deps, start_after, limit)
        }
        QueryMsg::ListSubDaos { start_after, limit } => {
            dao_dao_core::contract::query_list_sub_daos(deps, start_after, limit)
        }
        QueryMsg::DaoURI {} => dao_dao_core::contract::query_dao_uri(deps),

    }
    .map_err(Into::into)
}

/// This just stores the result for future query
#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> DaoProxyResult {

    match msg{
        Reply {
            id: RESPONSE_REPLY_ID,
            result: SubMsgResult::Ok(_),
        } => reply::forward_response_data(msg).map_err(Into::into),
        _=> dao_dao_core::contract::reply(deps, env, msg).map_err(Into::into),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract;
    use cosmwasm_std::testing::*;
    use speculoos::prelude::*;

    mod migrate {
        use super::*;
        use abstract_core::AbstractError;

        #[test]
        fn disallow_same_version() -> DaoProxyResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg { 
                abstract_account_id: 1, 
                ans_host_address: "ANS_HOST".to_string()
            });

            assert_that!(res).is_err().is_equal_to(DaoProxyError::Abstract(
                AbstractError::CannotDowngradeContract {
                    contract: PROXY.to_string(),
                    from: version.clone(),
                    to: version,
                },
            ));

            Ok(())
        }

        #[test]
        fn disallow_downgrade() -> DaoProxyResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            let big_version = "999.999.999";
            cw2::set_contract_version(deps.as_mut().storage, PROXY, big_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg { 
                abstract_account_id: 1, 
                ans_host_address: "ANS_HOST".to_string()
            });

            assert_that!(res).is_err().is_equal_to(DaoProxyError::Abstract(
                AbstractError::CannotDowngradeContract {
                    contract: PROXY.to_string(),
                    from: big_version.parse().unwrap(),
                    to: version,
                },
            ));

            Ok(())
        }

        #[test]
        fn disallow_name_change() -> DaoProxyResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            let old_version = "0.0.0";
            let old_name = "old:contract";
            cw2::set_contract_version(deps.as_mut().storage, old_name, old_version)?;

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg { 
                abstract_account_id: 1, 
                ans_host_address: "ANS_HOST".to_string()
            });

            assert_that!(res).is_err().is_equal_to(DaoProxyError::Abstract(
                AbstractError::ContractNameMismatch {
                    from: old_name.parse().unwrap(),
                    to: PROXY.parse().unwrap(),
                },
            ));

            Ok(())
        }

        #[test]
        fn works() -> DaoProxyResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            let small_version = "0.0.0";
            cw2::set_contract_version(deps.as_mut().storage, PROXY, small_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg { 
                abstract_account_id: 1, 
                ans_host_address: "ANS_HOST".to_string()
            })?;
            assert_that!(res.messages).has_length(0);

            assert_that!(cw2::get_contract_version(&deps.storage)?.version)
                .is_equal_to(version.to_string());
            Ok(())
        }
    }
}
