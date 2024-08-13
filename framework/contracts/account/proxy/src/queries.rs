use abstract_sdk::{
    std::{
        objects::AssetEntry,
        proxy::{
            state::{ANS_HOST, STATE},
            ConfigResponse,
        },
    },
    Resolve,
};
use abstract_std::proxy::HoldingAmountResponse;
use cosmwasm_std::{Addr, Deps, Env, StdResult};

use crate::contract::ProxyResult;

/// Returns the whitelisted modules
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = STATE.load(deps.storage)?;
    let modules: Vec<Addr> = state.modules;
    let resp = ConfigResponse {
        modules: modules
            .iter()
            .map(|module| -> String { module.to_string() })
            .collect(),
    };
    Ok(resp)
}

pub fn query_holding_amount(
    deps: Deps,
    env: Env,
    identifier: AssetEntry,
) -> ProxyResult<HoldingAmountResponse> {
    let ans_host = ANS_HOST.load(deps.storage)?;
    let asset_info = identifier.resolve(&deps.querier, &ans_host)?;
    Ok(HoldingAmountResponse {
        amount: asset_info.query_balance(&deps.querier, env.contract.address)?,
    })
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use super::*;

    use crate::contract::{execute, instantiate, query};
    use abstract_std::proxy::{ExecuteMsg, InstantiateMsg};
    use abstract_testing::prelude::*;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info, MockApi},
        DepsMut, OwnedDeps,
    };

    type MockDeps = OwnedDeps<MockStorage, MockApi, MockQuerier>;

    fn mock_init(deps: DepsMut) {
        let info = mock_info(OWNER, &[]);
        let msg = InstantiateMsg {
            account_id: TEST_ACCOUNT_ID,
            ans_host_address: TEST_ANS_HOST.to_string(),
            manager_addr: TEST_MANAGER.to_string(),
        };
        let _res = instantiate(deps, mock_env(), info, msg).unwrap();
    }

    pub fn execute_as_admin(deps: &mut MockDeps, msg: ExecuteMsg) -> ProxyResult {
        let info = mock_info(TEST_MANAGER, &[]);
        execute(deps.as_mut(), mock_env(), info, msg)
    }

    #[test]
    fn query_config() {
        let mut deps = mock_dependencies();
        deps.querier = MockAnsHost::new().with_defaults().to_querier();
        mock_init(deps.as_mut());
        execute_as_admin(
            &mut deps,
            ExecuteMsg::AddModules {
                modules: vec!["test_module".to_string()],
            },
        )
        .unwrap();

        let config: ConfigResponse = from_json(
            query(
                deps.as_ref(),
                mock_env(),
                abstract_std::proxy::QueryMsg::Config {},
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            config,
            ConfigResponse {
                modules: vec!["manager_address".to_string(), "test_module".to_string()],
            }
        );
    }
}
