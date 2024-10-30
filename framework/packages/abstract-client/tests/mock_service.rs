use abstract_interface::{RegisteredModule, ServiceDeployer};
use abstract_std::objects::dependency::StaticDependency;
use abstract_unit_test_utils::prelude::TEST_VERSION;
use cosmwasm_std::{to_json_binary, Empty};
use cw_orch::{contract::Contract, prelude::*};

pub const MODULE_ID: &str = "tester:service";

#[cosmwasm_schema::cw_serde]
pub struct MockMsg {}

pub fn mock_instantiate(
    _deps: ::cosmwasm_std::DepsMut,
    _env: ::cosmwasm_std::Env,
    _info: ::cosmwasm_std::MessageInfo,
    _msg: MockMsg,
) -> ::cosmwasm_std::StdResult<::cosmwasm_std::Response> {
    Ok(::cosmwasm_std::Response::new())
}

/// Execute entrypoint
pub fn mock_execute(
    _deps: ::cosmwasm_std::DepsMut,
    _env: ::cosmwasm_std::Env,
    _info: ::cosmwasm_std::MessageInfo,
    _msg: MockMsg,
) -> ::cosmwasm_std::StdResult<::cosmwasm_std::Response> {
    Ok(::cosmwasm_std::Response::new().set_data(b"test"))
}

/// Query entrypoint
pub fn mock_query(
    _deps: ::cosmwasm_std::Deps,
    _env: ::cosmwasm_std::Env,
    _msg: MockMsg,
) -> ::cosmwasm_std::StdResult<::cosmwasm_std::Binary> {
    Ok(to_json_binary("test").unwrap())
}

#[cw_orch::interface(MockMsg, MockMsg, MockMsg, Empty)]
pub struct MockService;

impl<T: ::cw_orch::prelude::CwEnv> ::cw_orch::prelude::Uploadable for MockService<T> {
    fn wrapper() -> <Mock as ::cw_orch::environment::TxHandler>::ContractSource {
        Box::new(ContractWrapper::<MockMsg, _, _, _, _, _>::new_with_empty(
            self::mock_execute,
            self::mock_instantiate,
            self::mock_query,
        ))
    }
}

#[allow(unused)]
impl<Chain: ::cw_orch::environment::CwEnv> MockService<Chain> {
    pub fn new_test(chain: Chain) -> Self {
        Self(::cw_orch::contract::Contract::new(MODULE_ID, chain))
    }
}

impl<Chain> RegisteredModule for MockService<Chain> {
    type InitMsg = MockMsg;

    fn module_id<'a>() -> &'a str {
        MODULE_ID
    }

    fn module_version<'a>() -> &'a str {
        TEST_VERSION
    }

    fn dependencies<'a>() -> &'a [StaticDependency] {
        &[]
    }
}

impl<Chain> From<Contract<Chain>> for MockService<Chain> {
    fn from(value: Contract<Chain>) -> Self {
        Self(value)
    }
}

impl<Chain: CwEnv> ServiceDeployer<Chain> for MockService<Chain> {}
