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
    #![allow(clippy::needless_borrows_for_generic_args)]
    use super::*;

    use crate::contract::{execute, instantiate, query, ProxyResult};
    use abstract_std::proxy::{ExecuteMsg, InstantiateMsg};
    use abstract_testing::prelude::*;
    use cosmwasm_std::{
        testing::{message_info, mock_dependencies, mock_env, MockApi},
        OwnedDeps,
    };

    type MockDeps = OwnedDeps<MockStorage, MockApi, MockQuerier>;

    fn mock_init(deps: &mut MockDeps) {
        let abstr = AbstractMockAddrs::new(deps.api);
        let info = message_info(&abstr.owner, &[]);
        let msg = InstantiateMsg {
            account_id: TEST_ACCOUNT_ID,
            ans_host_address: abstr.ans_host.to_string(),
            manager_addr: abstr.account.manager.to_string(),
        };
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    }

    pub fn execute_as_admin(deps: &mut MockDeps, msg: ExecuteMsg) -> ProxyResult {
        let abstr = AbstractMockAddrs::new(deps.api);
        let info = message_info(&abstr.account.manager, &[]);
        execute(deps.as_mut(), mock_env(), info, msg)
    }

    #[test]
    fn query_config() {
        let mut deps = mock_dependencies();
        deps.querier = MockAnsHost::new(deps.api).with_defaults().to_querier();
        mock_init(&mut deps);
        let abstr = AbstractMockAddrs::new(deps.api);

        execute_as_admin(
            &mut deps,
            ExecuteMsg::AddModules {
                modules: vec![abstr.module_address.to_string()],
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
                modules: vec![
                    abstr.account.manager.to_string(),
                    abstr.module_address.to_string()
                ],
            }
        );
    }
}
