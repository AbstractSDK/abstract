use abstract_sdk::{base::ModuleIbcEndpoint, features::AbstractRegistryAccess};
use abstract_std::{
    objects::module::{ModuleInfo, ModuleVersion},
    IBC_HOST,
};
use cosmwasm_std::Addr;

use crate::{state::ContractError, AppContract};

impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        SudoMsg,
    > ModuleIbcEndpoint
    for AppContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, CustomMigrateMsg, SudoMsg>
{
    fn ibc_host(
        &self,
        deps: cosmwasm_std::Deps,
        env: &cosmwasm_std::Env,
    ) -> Result<Addr, Self::Error> {
        let registry_query_result = self
            .abstract_registry(deps, env)?
            .query_module(
                ModuleInfo::from_id(
                    IBC_HOST,
                    ModuleVersion::from(abstract_ibc_host::contract::CONTRACT_VERSION),
                )?,
                &deps.querier,
            )
            .map_err(Into::<abstract_std::AbstractError>::into)?;

        Ok(registry_query_result.reference.unwrap_native()?)
    }
}

#[cfg(test)]
mod test {
    use abstract_sdk::base::ModuleIbcEndpoint;
    use abstract_std::native_addrs;
    use abstract_testing::mock_env_validated;
    use cosmwasm_std::Api;

    use crate::mock::{mock_init, BASIC_MOCK_APP};

    #[coverage_helper::test]
    fn ibc_host_address() {
        let deps = mock_init();
        let env = mock_env_validated(deps.api);
        let ibc_host = BASIC_MOCK_APP.ibc_host(deps.as_ref(), &env);

        let hrp = native_addrs::hrp_from_env(&env);
        let expected_ibc_host_canon = native_addrs::ibc_host_address(hrp, &deps.api).unwrap();
        let expected_ibc_host = deps.api.addr_humanize(&expected_ibc_host_canon).unwrap();

        assert_eq!(ibc_host, Ok(expected_ibc_host))
    }
}
