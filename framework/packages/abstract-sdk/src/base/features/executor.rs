use abstract_std::proxy;
use cosmwasm_std::{wasm_execute, Coin, CosmosMsg, Deps};

use crate::AbstractSdkResult;

use super::AccountIdentification;

/// Trait for modules that are allowed to execute on the proxy.
pub trait AccountExecutor: AccountIdentification {
    /// Execute proxy method on proxy contract
    fn execute_on_account(
        &self,
        deps: Deps,
        msg: &proxy::ExecuteMsg,
        funds: Vec<Coin>,
    ) -> AbstractSdkResult<CosmosMsg> {
        let account_address = self.account(deps)?;
        wasm_execute(account_address.into_addr(), msg, funds)
            .map(Into::into)
            .map_err(Into::into)
    }
}
