//! Mutable Client
//!
//! This module implements methods that are applied for test environments.
//! For more details see [`MutCwEnv`]

use cosmwasm_std::{Addr, Coin};
use cw_orch::environment::MutCwEnv;

use crate::{
    client::{AbstractClient, AbstractClientResult},
    infrastructure::Environment,
};

impl<Chain: MutCwEnv> AbstractClient<Chain> {
    /// Set balance for an address
    pub fn set_balance(&self, address: &Addr, amount: Vec<Coin>) -> AbstractClientResult<()> {
        self.environment()
            .set_balance(address, amount)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    /// Set on chain balance of addresses
    pub fn set_balances<'a>(
        &self,
        balances: impl IntoIterator<Item = (&'a Addr, Vec<Coin>)>,
    ) -> AbstractClientResult<()> {
        balances
            .into_iter()
            .try_for_each(|(address, amount)| self.set_balance(address, amount))?;
        Ok(())
    }

    /// Add balance for the address
    pub fn add_balance(&self, address: &Addr, amount: Vec<Coin>) -> AbstractClientResult<()> {
        self.environment()
            .add_balance(address, amount)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    /// Add balance for the addresses
    pub fn add_balances<'a>(
        &self,
        balances: impl IntoIterator<Item = (&'a Addr, Vec<Coin>)>,
    ) -> AbstractClientResult<()> {
        balances
            .into_iter()
            .try_for_each(|(address, amount)| self.add_balance(address, amount))
    }
}
