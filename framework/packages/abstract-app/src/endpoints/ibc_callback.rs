use abstract_sdk::features::AbstractRegistryAccess;
use abstract_std::{
    objects::module::{ModuleInfo, ModuleVersion},
    IBC_CLIENT,
};
use cosmwasm_std::Addr;

use crate::{state::ContractError, AppContract, IbcCallbackEndpoint};

impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        SudoMsg,
    > IbcCallbackEndpoint
    for AppContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, CustomMigrateMsg, SudoMsg>
{
    fn ibc_client_addr(
        &self,
        deps: cosmwasm_std::Deps,
        env: &cosmwasm_std::Env,
    ) -> Result<Addr, Self::Error> {
        let registry_query_result = self
            .abstract_registry(deps, env)?
            .query_module(
                ModuleInfo::from_id(
                    IBC_CLIENT,
                    ModuleVersion::from(abstract_ibc_client::contract::CONTRACT_VERSION),
                )?,
                &deps.querier,
            )
            .map_err(Into::<abstract_std::AbstractError>::into)?;

        Ok(registry_query_result.reference.unwrap_native()?)
    }
}

#[cfg(test)]
mod test {
    use abstract_sdk::base::IbcCallbackEndpoint;
    use abstract_std::{account::state::ACCOUNT_MODULES, IBC_CLIENT};
    use abstract_testing::{
        abstract_mock_querier_builder, mock_env_validated, prelude::test_account,
    };
    use cosmwasm_std::Addr;

    use crate::mock::{mock_init, BASIC_MOCK_APP};

    #[coverage_helper::test]
    fn ibc_client_address() {
        let mut deps = mock_init();
        let test_account = test_account(deps.api);
        let ibc_client_addr = Addr::unchecked("ibc_client");

        deps.querier = abstract_mock_querier_builder(deps.api)
            .with_contract_map_entry(
                test_account.addr(),
                ACCOUNT_MODULES,
                (IBC_CLIENT, ibc_client_addr.clone()),
            )
            .build();
        let env = mock_env_validated(deps.api);

        let ibc_client = BASIC_MOCK_APP.ibc_client_addr(deps.as_ref(), &env);
        assert_eq!(ibc_client, Ok(ibc_client_addr))
    }
}
