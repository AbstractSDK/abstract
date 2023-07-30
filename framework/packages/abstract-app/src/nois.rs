use crate::{state::ContractError, AppContract};
use abstract_core::objects::UncheckedContractEntry;
use abstract_sdk::base::NoisHandler;
use abstract_sdk::{features::AbstractNameService, AbstractSdkResult};

use abstract_sdk::NoisInterface;
use cosmwasm_std::{Addr, Deps};

const NOIS_PROTOCOL: &str = "nois";
const NOIS_PROXY_CONTRACT: &str = "proxy";

impl<Error: ContractError, InitMsg, ExecMsg, QueryMsg, MigrateMsg, ReceiveMsg, SudoMsg> NoisHandler
    for AppContract<Error, InitMsg, ExecMsg, QueryMsg, MigrateMsg, ReceiveMsg, SudoMsg>
{
}

impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    > NoisInterface
    for AppContract<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
{
    fn nois_proxy_address(&self, deps: Deps) -> AbstractSdkResult<Addr> {
        let ans_host = self.ans_host(deps)?;

        let nois_proxy = ans_host.query_contract(
            &deps.querier,
            &UncheckedContractEntry::new(NOIS_PROTOCOL, NOIS_PROXY_CONTRACT).check(),
        )?;

        Ok(nois_proxy)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use abstract_core::objects::ContractEntry;
    use abstract_sdk::features::ModuleIdentification;
    use abstract_testing::prelude::{TEST_ANS_HOST, TEST_MODULE_ID, TEST_PROXY};
    use speculoos::prelude::*;

    use crate::mock::*;
    use abstract_testing::{prelude::*, MockAnsHost};

    #[test]
    fn test_nois_proxy_address() -> AppTestResult {
        let mut deps = mock_init();

        let mut mock_ans = MockAnsHost::new();
        mock_ans.contracts.append((
            ContractEntry {
                protocol: NOIS_PROTOCOL,
                contract: NOIS_PROXY_CONTRACT,
            },
            "proxy_addr",
        ));

        deps.querier = mock_ans.to_querier();

        assert_that!(MOCK_APP.nois_proxy_address(deps.as_ref())).is_equal_to("proxy_addr");

        Ok(())
    }
}
