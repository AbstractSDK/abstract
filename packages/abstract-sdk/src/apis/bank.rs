//! # Bank
//! The Bank object handles asset transfers to and from the OS.

use abstract_os::objects::AnsAsset;
use cosmwasm_std::{Addr, BankMsg, Coin, CosmosMsg, Deps, StdResult};

use super::{execution::Execution, AbstractNameService};
use crate::ans_resolve::Resolve;

/// Bank assets from and to the Abstract OS.
pub trait TransferInterface: AbstractNameService + Execution {
    fn bank<'a>(&'a self, deps: Deps<'a>) -> Bank<Self> {
        Bank { base: self, deps }
    }
}

impl<T> TransferInterface for T where T: AbstractNameService + Execution {}

#[derive(Clone)]
pub struct Bank<'a, T: TransferInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: TransferInterface> Bank<'a, T> {
    /// Bank funds from the OS
    pub fn transfer(&self, funds: Vec<AnsAsset>, to_address: &Addr) -> StdResult<CosmosMsg> {
        let resolved_funds = funds.resolve(&self.deps.querier, &self.base.ans_host(self.deps)?)?;
        let transfer_msgs = resolved_funds
            .iter()
            .map(|asset| asset.transfer_msg(to_address.clone()))
            .collect::<StdResult<Vec<CosmosMsg>>>();
        self.base.executor(self.deps).execute(transfer_msgs?)
    }

    /// Deposit into the OS from the current contract
    pub fn deposit(&self, funds: Vec<AnsAsset>) -> StdResult<Vec<CosmosMsg>> {
        let to = self.base.proxy_address(self.deps)?;
        let resolved_funds = funds.resolve(&self.deps.querier, &self.base.ans_host(self.deps)?)?;
        resolved_funds
            .iter()
            .map(|asset| asset.transfer_msg(to.clone()))
            .collect::<StdResult<Vec<CosmosMsg>>>()
    }

    /// Deposit coins into the OS
    pub fn deposit_coins(&self, coins: Vec<Coin>) -> StdResult<CosmosMsg> {
        let to_address = self.base.proxy_address(self.deps)?.into_string();
        Ok(CosmosMsg::Bank(BankMsg::Send {
            to_address,
            amount: coins,
        }))
    }

    /// Transfer coins from the OS
    pub fn transfer_coins(&self, coins: Vec<Coin>, to_address: &Addr) -> StdResult<CosmosMsg> {
        let send_msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: to_address.to_string(),
            amount: coins,
        });
        self.base.executor(self.deps).execute(vec![send_msg])
    }
}
