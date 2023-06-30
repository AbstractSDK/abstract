#![allow(unused)]
use crate::AccountAction;
use crate::{AbstractSdkResult, TransferInterface};
use abstract_core::objects::AnsAsset;
use cosmwasm_std::{Addr, CosmosMsg, Deps, StdResult, Uint128};
// ANCHOR: splitter
// Trait to retrieve the Splitter object
// Depends on the ability to transfer funds
pub trait SplitterInterface: TransferInterface {
    fn splitter<'a>(&'a self, deps: Deps<'a>) -> Splitter<Self> {
        Splitter { base: self, deps }
    }
}

// Implement for every object that can transfer funds
impl<T> SplitterInterface for T where T: TransferInterface {}

#[derive(Clone)]
pub struct Splitter<'a, T: SplitterInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: SplitterInterface> Splitter<'a, T> {
    /// Split an asset to multiple users
    pub fn split(&self, asset: AnsAsset, receivers: &[Addr]) -> AbstractSdkResult<AccountAction> {
        // split the asset between all receivers
        let receives_each = AnsAsset {
            amount: asset
                .amount
                .multiply_ratio(Uint128::one(), Uint128::from(receivers.len() as u128)),
            ..asset
        };

        // Retrieve the bank API
        let bank = self.base.bank(self.deps);
        receivers
            .iter()
            .map(|receiver| {
                // Construct the transfer message
                bank.transfer(vec![&receives_each], receiver)
            })
            .try_fold(AccountAction::new(), |mut acc, v| match v {
                Ok(action) => {
                    // Merge two AccountAction objects
                    acc.merge(action);
                    Ok(acc)
                }
                Err(e) => Err(e),
            })
    }
}
// ANCHOR_END: splitter

#[cfg(test)]
mod test {
    use abstract_core::objects::AnsAsset;
    use cosmwasm_std::{testing::mock_dependencies, Addr, CosmosMsg, Response, StdError, Uint128};

    use crate::{
        apis::splitter::SplitterInterface, mock_module::MockModule, AbstractSdkError, Execution,
    };

    fn split() -> Result<Response, AbstractSdkError> {
        let deps = mock_dependencies();
        let module = MockModule::new();
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

        let split_funds = module.splitter(deps.as_ref()).split(asset, &receivers)?;
        assert_eq!(split_funds.messages().len(), 3);

        let msg: CosmosMsg = module.executor(deps.as_ref()).execute(vec![split_funds])?;

        Ok(Response::new().add_message(msg))
        // ANCHOR_END: usage
    }
}
