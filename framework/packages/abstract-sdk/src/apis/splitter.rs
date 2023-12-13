#![allow(unused)]
use super::{AbstractApi, ApiIdentification};
use crate::features::ModuleIdentification;
use crate::AccountAction;
use crate::{AbstractSdkResult, TransferInterface};
use abstract_core::objects::AnsAsset;
use cosmwasm_std::{Addr, CosmosMsg, Deps, StdResult, Uint128};
// ANCHOR: splitter
// Trait to retrieve the Splitter object
// Depends on the ability to transfer funds
pub trait SplitterInterface: TransferInterface + ModuleIdentification {
    fn splitter<'a>(&'a mut self) -> Splitter<Self> {
        Splitter { base: self }
    }
}

// Implement for every object that can transfer funds
impl<T> SplitterInterface for T where T: TransferInterface + ModuleIdentification {}

impl<'a, T: SplitterInterface> AbstractApi<T> for Splitter<'a, T> {
    fn base(&self) -> &T {
        self.base
    }
}

impl<'a, T: SplitterInterface> ApiIdentification for Splitter<'a, T> {
    fn api_id() -> String {
        "Splitter".to_owned()
    }
}

pub struct Splitter<'a, T: SplitterInterface> {
    base: &'a mut T,
}

impl<'a, T: SplitterInterface> Splitter<'a, T> {
    /// Split an asset to multiple users
    pub fn split(&mut self, asset: AnsAsset, receivers: &[Addr]) -> AbstractSdkResult<()> {
        // split the asset between all receivers
        let receives_each = AnsAsset {
            amount: asset
                .amount
                .multiply_ratio(Uint128::one(), Uint128::from(receivers.len() as u128)),
            ..asset
        };

        // Retrieve the bank API
        let mut bank = self.base.bank();
        receivers.iter().try_for_each(|receiver| {
            // Construct the transfer message
            // TODO, ability to merge account actions ?
            bank.transfer(vec![&receives_each], receiver)
        })
    }
}
// ANCHOR_END: splitter

#[cfg(test)]
mod test {
    use abstract_core::objects::AnsAsset;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, CosmosMsg, Response, StdError, Uint128,
    };

    use crate::base::features::ResponseGenerator;
    use crate::{
        apis::splitter::SplitterInterface, mock_module::MockModule, AbstractSdkError, Execution,
        ExecutorMsg,
    };

    fn split() -> Result<Response, AbstractSdkError> {
        let mut deps = mock_dependencies();
        let mut module =
            MockModule::new((deps.as_mut(), mock_env(), mock_info("sender", &[])).into());
        // ANCHOR: usage
        let asset = AnsAsset {
            amount: Uint128::from(100u128),
            name: "usd".into(),
        };

        let receivers = vec![
            Addr::unchecked("receiver1"),
            Addr::unchecked("receiver2"),
            Addr::unchecked("receiver3"),
        ];

        module.splitter().split(asset, &receivers)?;

        Ok(module._generate_response()?)
        // ANCHOR_END: usage
    }
}
