use abstract_macros::abstract_response;
use abstract_sdk::{
    feature_objects::AnsHost,
    std::{
        objects::account::ACCOUNT_ID,
        proxy::{
            state::{State, ADMIN, ANS_HOST, STATE},
            AssetConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
        },
        PROXY,
    },
};
use abstract_std::objects::{
    module_version::assert_contract_upgrade, oracle::Oracle, price_source::UncheckedPriceSource,
};
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, SubMsgResult,
};
use semver::Version;

use crate::{commands::*, error::ProxyError, queries::*, reply};

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const RESPONSE_REPLY_ID: u64 = 1;

#[abstract_response(PROXY)]
pub struct ProxyResponse;

/// The result type for the proxy contract.
pub type ProxyResult<T = Response> = Result<T, ProxyError>;

/*
    The proxy is the bank account of the account. It owns the liquidity and acts as a proxy contract.
    Whitelisted dApps construct messages for this contract. The dApps are controlled by the Manager.
*/

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ProxyResult {
    // Use CW2 to set the contract version, this is needed for migrations
    cw2::set_contract_version(deps.storage, PROXY, CONTRACT_VERSION)?;

    let manager_addr = deps.api.addr_validate(&msg.manager_addr)?;
    ACCOUNT_ID.save(deps.storage, &msg.account_id)?;
    STATE.save(
        deps.storage,
        &State {
            modules: vec![manager_addr.clone()],
        },
    )?;
    let ans_host = AnsHost {
        address: deps.api.addr_validate(&msg.ans_host_address)?,
    };
    ANS_HOST.save(deps.storage, &ans_host)?;
    let admin_addr = Some(manager_addr);
    ADMIN.set(deps.branch(), admin_addr)?;

    if let Some(base_asset) = msg.base_asset {
        let oracle = Oracle::new();
        oracle.update_assets(
            deps,
            &ans_host,
            vec![(base_asset, UncheckedPriceSource::None)],
            vec![],
        )?;
    }
    Ok(Response::default())
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> ProxyResult {
    match msg {
        ExecuteMsg::ModuleAction { msgs } => execute_module_action(deps, info, msgs),
        ExecuteMsg::ModuleActionWithData { msg } => execute_module_action_response(deps, info, msg),
        ExecuteMsg::IbcAction { msgs } => execute_ibc_action(deps, info, msgs),
        ExecuteMsg::SetAdmin { admin } => set_admin(deps, info, &admin),
        ExecuteMsg::AddModules { modules } => add_modules(deps, info, modules),
        ExecuteMsg::RemoveModule { module } => remove_module(deps, info, module),
        ExecuteMsg::UpdateAssets { to_add, to_remove } => {
            update_assets(deps, info, to_add, to_remove)
        }
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ProxyResult {
    let version: Version = CONTRACT_VERSION.parse().unwrap();

    assert_contract_upgrade(deps.storage, PROXY, version)?;
    cw2::set_contract_version(deps.storage, PROXY, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> ProxyResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::TotalValue {} => to_json_binary(&query_total_value(deps, env)?),
        QueryMsg::HoldingAmount { identifier } => {
            to_json_binary(&query_holding_amount(deps, env, identifier)?)
        }
        QueryMsg::TokenValue { identifier } => {
            to_json_binary(&query_token_value(deps, env, identifier)?)
        }
        QueryMsg::AssetConfig { identifier } => to_json_binary(&AssetConfigResponse {
            price_source: Oracle::new().asset_config(deps, &identifier)?,
        }),
        QueryMsg::AssetsConfig { start_after, limit } => {
            to_json_binary(&query_oracle_asset_config(deps, start_after, limit)?)
        }
        QueryMsg::AssetsInfo { start_after, limit } => {
            to_json_binary(&query_oracle_asset_info(deps, start_after, limit)?)
        }
        QueryMsg::BaseAsset {} => to_json_binary(&query_base_asset(deps)?),
    }
    .map_err(Into::into)
}

/// This just stores the result for future query
#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> ProxyResult {
    match &msg {
        Reply {
            id: RESPONSE_REPLY_ID,
            result: SubMsgResult::Ok(_),
        } => reply::forward_response_data(msg),
        _ => Err(ProxyError::UnexpectedReply {}),
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::*;
    use speculoos::prelude::*;

    use super::*;
    use crate::{contract, test_common::*};

    mod migrate {
        use abstract_std::AbstractError;

        use super::*;

        #[test]
        fn disallow_same_version() -> ProxyResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res).is_err().is_equal_to(ProxyError::Abstract(
                AbstractError::CannotDowngradeContract {
                    contract: PROXY.to_string(),
                    from: version.clone(),
                    to: version,
                },
            ));

            Ok(())
        }

        #[test]
        fn disallow_downgrade() -> ProxyResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            let big_version = "999.999.999";
            cw2::set_contract_version(deps.as_mut().storage, PROXY, big_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res).is_err().is_equal_to(ProxyError::Abstract(
                AbstractError::CannotDowngradeContract {
                    contract: PROXY.to_string(),
                    from: big_version.parse().unwrap(),
                    to: version,
                },
            ));

            Ok(())
        }

        #[test]
        fn disallow_name_change() -> ProxyResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            let old_version = "0.0.0";
            let old_name = "old:contract";
            cw2::set_contract_version(deps.as_mut().storage, old_name, old_version)?;

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res).is_err().is_equal_to(ProxyError::Abstract(
                AbstractError::ContractNameMismatch {
                    from: old_name.parse().unwrap(),
                    to: PROXY.parse().unwrap(),
                },
            ));

            Ok(())
        }

        #[test]
        fn works() -> ProxyResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let small_version = Version {
                minor: version.minor - 1,
                ..version.clone()
            }
            .to_string();
            cw2::set_contract_version(deps.as_mut().storage, PROXY, small_version)?;

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {})?;
            assert_that!(res.messages).has_length(0);

            assert_that!(cw2::get_contract_version(&deps.storage)?.version)
                .is_equal_to(version.to_string());
            Ok(())
        }
    }
}
