use std::collections::BTreeMap;

use cosmwasm_std::{from_json, to_json_binary, Addr, Binary};
use serde_cw_value::Value;

use super::IbcMemoBuilder;

/// Builder for [IbcHooks](https://github.com/cosmos/ibc-apps/tree/main/modules/ibc-hooks) memo field.
pub struct IbcHooksBuilder {
    contract_addr: Addr,
    msg: Binary,
    ibc_callback: Option<Addr>,
}

impl IbcHooksBuilder {
    /// New Wasm Contract Memo IBC Hook
    pub fn new(contract_addr: Addr, msg: &impl serde::Serialize) -> Self {
        let msg = to_json_binary(&msg).unwrap();
        Self {
            contract_addr,
            msg,
            ibc_callback: None,
        }
    }

    /// Contract that will receive callback, see:
    /// https://github.com/cosmos/ibc-apps/blob/main/modules/ibc-hooks/README.md#interface-for-receiving-the-acks-and-timeouts
    pub fn callback_contract(mut self, callback_contract: Addr) -> Self {
        self.ibc_callback = Some(callback_contract);
        self
    }
}

impl IbcMemoBuilder for IbcHooksBuilder {
    fn build_value_map(self) -> BTreeMap<Value, Value> {
        let execute_wasm_value = BTreeMap::from([
            (
                Value::String("contract".to_owned()),
                Value::String(self.contract_addr.into_string()),
            ),
            (
                Value::String("msg".to_owned()),
                from_json(&self.msg).expect("expected valid json message"),
            ),
        ]);

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
        memo
    }
}
