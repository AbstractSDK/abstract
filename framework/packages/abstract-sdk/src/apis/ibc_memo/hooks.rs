use std::collections::BTreeMap;

use cosmwasm_std::{from_json, to_json_binary, Addr, Binary, Coin};
use serde_cw_value::Value;

use super::IbcMemoBuilder;

/// Builder for [IbcHooks](https://github.com/cosmos/ibc-apps/tree/main/modules/ibc-hooks) memo field.
pub struct IbcHooksBuilder {
    contract_addr: Addr,
    msg: Binary,
    funds: Option<Vec<Coin>>,
    ibc_callback: Option<Addr>,
}

impl IbcHooksBuilder {
    /// New Wasm Contract Memo IBC Hook
    pub fn new(contract_addr: Addr, msg: &impl serde::Serialize) -> Self {
        let msg = to_json_binary(&msg).unwrap();
        Self {
            contract_addr,
            msg,
            funds: None,
            ibc_callback: None,
        }
    }

    /// Add funds to hook
    pub fn funds(mut self, funds: Vec<Coin>) -> Self {
        self.funds = Some(funds);
        self
    }

    /// Contract that will receive callback, see:
    /// https://github.com/cosmos/ibc-apps/blob/main/modules/ibc-hooks/README.md#interface-for-receiving-the-acks-and-timeouts
    pub fn callback_contract(mut self, callback_contract: Addr) -> Self {
        self.ibc_callback = Some(callback_contract);
        self
    }
}

impl IbcMemoBuilder for IbcHooksBuilder {
    fn build_value(self) -> Value {
        let mut execute_wasm_value = BTreeMap::from([
            (
                Value::String("contract".to_owned()),
                Value::String(self.contract_addr.into_string()),
            ),
            (
                Value::String("msg".to_owned()),
                from_json(&self.msg).expect("expected valid json message"),
            ),
        ]);

        if let Some(funds) = self.funds {
            execute_wasm_value.insert(
                Value::String("funds".to_owned()),
                Value::Seq(
                    funds
                        .into_iter()
                        .map(|coin| {
                            Value::Map(BTreeMap::from([
                                (Value::String("denom".to_owned()), Value::String(coin.denom)),
                                (
                                    Value::String("amount".to_owned()),
                                    Value::String(coin.amount.to_string()),
                                ),
                            ]))
                        })
                        .collect(),
                ),
            );
        }

        let mut memo = BTreeMap::from([(
            Value::String("wasm".to_owned()),
            Value::Map(execute_wasm_value.into_iter().collect()),
        )]);
        if let Some(contract_addr) = self.ibc_callback {
            memo.insert(
                Value::String("ibc_callback".to_owned()),
                Value::String(contract_addr.into_string()),
            );
        }
        Value::Map(memo)
    }
}
