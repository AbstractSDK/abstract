use abstract_core::abstract_token::*;
use cosmwasm_std::{Addr, Binary, Uint128};
use cw_orch::{interface, prelude::*};

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct Idea<Chain>;

impl<Chain: CwEnv> Idea<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        Self(cw_orch::contract::Contract::new(name, chain))
    }
    pub fn send(
        &self,
        msg: Binary,
        amount: u128,
        contract: String,
    ) -> Result<TxResponse<Chain>, crate::AbstractInterfaceError> {
        let msg = ExecuteMsg::Send {
            contract,
            amount: Uint128::new(amount),
            msg,
        };

        self.execute(&msg, None).map_err(Into::into)
    }

    /// Instantiate a new token instance with some initial balance given to the minter
    pub fn create_new<T: Into<Uint128>>(
        &self,
        minter: &Addr,
        balance: T,
        version_control_address: String,
        symbol: &str,
    ) -> Result<TxResponse<Chain>, crate::AbstractInterfaceError> {
        let msg = InstantiateMsg {
            decimals: 6,
            mint: Some(MinterResponse {
                cap: None,
                minter: minter.clone().into(),
            }),
            symbol: symbol.to_string(),
            name: self.as_instance().id.to_string(),
            initial_balances: vec![Cw20Coin {
                address: minter.clone().into(),
                amount: balance.into(),
            }],
            version_control_address,
        };

        self.instantiate(&msg, Some(minter), None)
            .map_err(Into::into)
    }
}
