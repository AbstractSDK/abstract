use abstract_sdk::std::proxy::{state::STATE, ConfigResponse};
use cosmwasm_std::{Addr, Deps, StdResult};

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

#[cfg(test)]
mod test {
    use abstract_std::proxy::{ExecuteMsg, InstantiateMsg};
    use abstract_testing::{prelude::*, MockAnsHost};
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage},
        DepsMut, OwnedDeps,
    };

    use super::*;
    use crate::contract::{execute, instantiate, query, ProxyResult};

    type MockDeps = OwnedDeps<MockStorage, MockApi, MockQuerier>;

    fn mock_init(deps: DepsMut) {
        let info = mock_info(OWNER, &[]);
        let msg = InstantiateMsg {
            account_id: TEST_ACCOUNT_ID,
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
