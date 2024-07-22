use std::collections::BTreeMap;

use cosmwasm_std::{from_json, to_json_binary, Addr, Binary, Coin};
use serde_cw_value::Value;

/// Trait for memo-based IBC message builders.
pub trait IbcMemoBuilder {
    /// Build the memo json [Value] object.
    fn build_value(self) -> Value;
    /// Build the memo json string.
    fn build(self) -> cosmwasm_std::StdResult<String>
    where
        Self: Sized,
    {
        cosmwasm_std::to_json_string(&self.build_value())
    }
}

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

/// Builder for [Packet Forward Middleware](https://github.com/cosmos/ibc-apps/tree/main/middleware/packet-forward-middleware) memos.
pub struct PacketForwardMiddlewareBuilder {
    channel: String,
    receiver: Option<Addr>,
    port: Option<String>,
    timeout: Option<String>,
    retries: Option<u8>,
    next: Option<Value>,
}

impl PacketForwardMiddlewareBuilder {
    /// Create forward memo
    pub fn new(channel: impl Into<String>) -> Self {
        Self {
            channel: channel.into(),
            receiver: None,
            port: None,
            timeout: None,
            retries: None,
            next: None,
        }
    }

    /// Address of the receiver, defaults to `pfm`
    /// https://github.com/cosmos/ibc-apps/tree/main/middleware/packet-forward-middleware#intermediate-receivers
    pub fn receiver(mut self, receiver: Addr) -> Self {
        self.receiver = Some(receiver);
        self
    }

    /// Port, defaults to "transfer"
    pub fn port(mut self, port: impl Into<String>) -> Self {
        self.port = Some(port.into());
        self
    }

    /// Timeout duration, for example: "10m"
    pub fn timeout(mut self, timeout: impl Into<String>) -> Self {
        self.timeout = Some(timeout.into());
        self
    }

    /// Retries number
    pub fn retries(mut self, retries: u8) -> Self {
        self.retries = Some(retries);
        self
    }

    /// Add next memo to middleware
    pub fn next(mut self, next_memo: impl IbcMemoBuilder) -> Self {
        self.next = Some(next_memo.build_value());
        self
    }
}

impl IbcMemoBuilder for PacketForwardMiddlewareBuilder {
    fn build_value(self) -> Value {
        let PacketForwardMiddlewareBuilder {
            receiver,
            port,
            channel,
            timeout,
            retries,
            next,
        } = self;
        let receiver = receiver.map(Addr::into_string).unwrap_or("pfm".to_owned());
        let port = port.unwrap_or("transfer".to_owned());

        let mut forward_value = BTreeMap::from([
            (
                Value::String("receiver".to_owned()),
                Value::String(receiver),
            ),
            (Value::String("port".to_owned()), Value::String(port)),
            (Value::String("channel".to_owned()), Value::String(channel)),
        ]);
        if let Some(timeout) = timeout {
            forward_value.insert(Value::String("timeout".to_owned()), Value::String(timeout));
        }
        if let Some(retries) = retries {
            forward_value.insert(Value::String("retries".to_owned()), Value::U8(retries));
        }
        if let Some(next) = next {
            forward_value.insert(Value::String("next".to_owned()), next);
        }

        Value::Map(BTreeMap::from([(
            Value::String("forward".to_owned()),
            Value::Map(forward_value.into_iter().collect()),
        )]))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::coins;
    use serde_json::json;

    #[test]
    fn memo_middleware() {
        let minimal = PacketForwardMiddlewareBuilder::new("channel-1")
            .build()
            .unwrap();
        let value: serde_json::Value = serde_json::from_str(&minimal).unwrap();
        let expected_value = json!({
            "forward": {
                "channel": "channel-1",
                "port": "transfer",
                "receiver": "pfm",
            }
        });
        assert_eq!(value, expected_value);

        let complete = PacketForwardMiddlewareBuilder::new("channel-1")
            .receiver(Addr::unchecked("foo"))
            .port("different_port")
            .timeout("10m")
            .retries(4)
            .next(PacketForwardMiddlewareBuilder::new("channel-2"))
            .build()
            .unwrap();
        let value: serde_json::Value = serde_json::from_str(&complete).unwrap();
        let expected_value = json!({
            "forward": {
                "channel": "channel-1",
                "port": "different_port",
                "receiver": "foo",
                "timeout": "10m",
                "retries": 4,
                "next": {
                    "forward": {
                        "channel": "channel-2",
                        "port": "transfer",
                        "receiver": "pfm",
                    }
                }
            }
        });
        assert_eq!(value, expected_value);

        let multimultihop = PacketForwardMiddlewareBuilder::new("channel-1")
            .next(
                PacketForwardMiddlewareBuilder::new("channel-2")
                    .next(PacketForwardMiddlewareBuilder::new("channel-3")),
            )
            .build()
            .unwrap();
        let value: serde_json::Value = serde_json::from_str(&multimultihop).unwrap();
        let expected_value = json!({
            "forward": {
                "channel": "channel-1",
                "port": "transfer",
                "receiver": "pfm",
                "next": {
                    "forward": {
                        "channel": "channel-2",
                        "port": "transfer",
                        "receiver": "pfm",
                        "next": {
                            "forward": {
                                "channel": "channel-3",
                                "port": "transfer",
                                "receiver": "pfm",
                            }
                        }
                    }
                }
            }
        });
        assert_eq!(value, expected_value);
    }

    #[test]
    fn memo_wasm_hook() {
        let msg = json!({
            "withdraw": {}
        });

        let minimal = IbcHooksBuilder::new(Addr::unchecked("mock_addr"), &msg)
            .build()
            .unwrap();
        let value: serde_json::Value = serde_json::from_str(&minimal).unwrap();
        let expected_value = json!({
            "wasm": {
                "contract": "mock_addr",
                "msg": {"withdraw": {}}
            }
        });
        assert_eq!(value, expected_value);

        let complete = IbcHooksBuilder::new(Addr::unchecked("mock_addr"), &msg)
            .funds(coins(42, "abstract"))
            .callback_contract(Addr::unchecked("callback_addr"))
            .build()
            .unwrap();
        let value: serde_json::Value = serde_json::from_str(&complete).unwrap();
        let expected_value = json!({
            "wasm": {
                "contract": "mock_addr",
                "msg": {"withdraw": {}},
                "funds": [{
                    "amount": "42",
                    "denom": "abstract"
                }]
            },
            "ibc_callback": "callback_addr"
        });
        assert_eq!(value, expected_value);
    }

    #[test]
    fn memo_hop_wasm_hook() {
        let memo = PacketForwardMiddlewareBuilder::new("channel-1")
            .next(IbcHooksBuilder::new(
                Addr::unchecked("mock_addr"),
                &json!({
                    "withdraw": {}
                }),
            ))
            .build()
            .unwrap();
        let value: serde_json::Value = serde_json::from_str(&memo).unwrap();
        let expected_value = json!({
            "forward": {
                "channel": "channel-1",
                "port": "transfer",
                "receiver": "pfm",
                "next": {
                    "wasm": {
                        "contract": "mock_addr",
                        "msg": {"withdraw": {}}
                    }
                }
            }
        });
        assert_eq!(value, expected_value);
    }
}
