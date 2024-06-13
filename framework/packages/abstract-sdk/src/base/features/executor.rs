use abstract_std::proxy;
use cosmwasm_std::{wasm_execute, CosmosMsg, Deps};

use crate::AbstractSdkResult;

use super::AccountIdentification;

/// Trait for modules that are allowed to execute on the proxy.
pub trait AccountExecutor: AccountIdentification {
    /// Execute proxy method on proxy contract
    fn execute_on_proxy(
        &self,
        deps: Deps,
        msg: &proxy::ExecuteMsg,
    ) -> AbstractSdkResult<CosmosMsg> {
        let proxy_address = self.proxy_address(deps)?;
        wasm_execute(proxy_address, msg, vec![])
            .map(Into::into)
            .map_err(Into::into)
    }
}
