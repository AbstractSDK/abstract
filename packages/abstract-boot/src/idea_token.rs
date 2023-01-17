use abstract_os::abstract_token::*;
use boot_core::{
    interface::{BootExecute, BootInstantiate, ContractInstance},
    prelude::boot_contract,
};
use boot_core::{BootEnvironment, BootError, Contract, TxResponse};
use cosmwasm_std::{Addr, Binary, Uint128};

#[boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct Idea<Chain>;

impl<Chain: BootEnvironment> Idea<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        Self(
            Contract::new(name, chain).with_wasm_path("abstract_token"), // .with_mock(Box::new(
                                                                         //     ContractWrapper::new_with_empty(
                                                                         //         ::contract::execute,
                                                                         //         ::contract::instantiate,
                                                                         //         ::contract::query,
                                                                         //     ),
                                                                         // ))
        )
    }
    pub fn send(
        &self,
        msg: Binary,
        amount: u128,
        contract: String,
    ) -> Result<TxResponse<Chain>, BootError> {
        let msg = ExecuteMsg::Send {
            contract,
            amount: Uint128::new(amount),
            msg,
        };

        self.execute(&msg, None)
    }

    /// Instantiate a new token instance with some initial balance given to the minter
    pub fn create_new<T: Into<Uint128>>(
        &self,
        minter: &Addr,
        balance: T,
        version_control_address: String,
        symbol: &str,
    ) -> Result<TxResponse<Chain>, BootError> {
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
    }
}
