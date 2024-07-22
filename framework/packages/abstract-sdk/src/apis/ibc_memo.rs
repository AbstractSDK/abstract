mod hooks;
mod pfm;

pub use hooks::IbcHooksBuilder;
pub use pfm::PacketForwardMiddlewareBuilder;
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

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::{coins, Addr};
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
