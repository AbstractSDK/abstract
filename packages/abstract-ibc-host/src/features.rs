use crate::{state::ContractError, Host};
use abstract_core::objects::AccountId;
use abstract_sdk::{
    features::{AbstractNameService, AccountIdentification},
    AbstractSdkError, AbstractSdkResult,
};
use cosmwasm_std::Deps;

impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        SudoMsg,
        ReceiveMsg,
    > AbstractNameService
    for Host<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        SudoMsg,
        ReceiveMsg,
    >
{
    fn ans_host(&self, deps: Deps) -> AbstractSdkResult<abstract_sdk::feature_objects::AnsHost> {
        Ok(self.base_state.load(deps.storage)?.ans_host)
    }
}

impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        SudoMsg,
        ReceiveMsg,
    > AccountIdentification
    for Host<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        SudoMsg,
        ReceiveMsg,
    >
{
    fn proxy_address(&self, _deps: Deps) -> AbstractSdkResult<cosmwasm_std::Addr> {
        self.target()
            .map_err(|e| AbstractSdkError::generic_err(e.to_string()))
            .map(ToOwned::to_owned)
    }
    fn manager_address(&self, _deps: Deps) -> AbstractSdkResult<cosmwasm_std::Addr> {
        Err(AbstractSdkError::generic_err(
            "manager address not available on stateless ibc deployment",
        ))
    }

    fn account_base(
        &self,
        _deps: Deps,
    ) -> AbstractSdkResult<abstract_sdk::core::version_control::AccountBase> {
        Err(AbstractSdkError::generic_err(
            "Account base not available on stateless ibc deployment",
        ))
    }

    fn account_id(&self, _deps: Deps) -> AbstractSdkResult<AccountId> {
        Err(AbstractSdkError::generic_err(
            "account_id not available on stateless ibc deployment",
        ))
    }
}
