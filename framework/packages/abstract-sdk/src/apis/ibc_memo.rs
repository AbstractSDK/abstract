mod hooks;
mod pfm;
// mod pfm;

use std::collections::BTreeMap;

pub use hooks::IbcHooksBuilder;
pub use pfm::PacketForwardMiddlewareBuilder;
use serde_cw_value::Value;

/// Trait for memo-based IBC message builders.
pub trait IbcMemoBuilder {
    /// Build the memo json [Value] object.
    fn build_value_map(self) -> BTreeMap<Value, Value>;
    /// Build the memo json string.
    fn build(self) -> cosmwasm_std::StdResult<String>
    where
        Self: Sized,
    {
        cosmwasm_std::to_json_string(&self.build_value_map())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::Addr;
    use serde_json::json;

    #[test]
    fn memo_middleware() {
        let empty = PacketForwardMiddlewareBuilder::new("who").build().unwrap();
        let value: serde_json::Value = serde_json::from_str(&empty).unwrap();
        let expected_value = json!({});
        assert_eq!(value, expected_value);

        let minimal = PacketForwardMiddlewareBuilder::new("foo")
            .hop("channel-1")
            .build()
            .unwrap();
        let value: serde_json::Value = serde_json::from_str(&minimal).unwrap();
        let expected_value = json!({
            "forward": {
                "channel": "channel-1",
                "port": "transfer",
                "receiver": "foo",
            }
        });
        assert_eq!(value, expected_value);

        let complete = PacketForwardMiddlewareBuilder::new("foo")
            .port("different_port")
            .hop("channel-1")
            .timeout("10m")
            .retries(4)
            .hop("channel-2")
            .build()
            .unwrap();
        let value: serde_json::Value = serde_json::from_str(&complete).unwrap();
        let expected_value = json!({
            "forward": {
                "channel": "channel-1",
                "port": "different_port",
                "receiver": "pfm",
                "timeout": "10m",
                "retries": 4,
                "next": {
                    "forward": {
                        "channel": "channel-2",
                        "port": "different_port",
                        "receiver": "foo",
                    }
                }
            }
        });
        assert_eq!(value, expected_value);

        let multimultihop = PacketForwardMiddlewareBuilder::new("receiver")
            .hop("channel-1")
            .hop("channel-2")
            .hop("channel-3")
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
                                "receiver": "receiver",
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

        let minimal = IbcHooksBuilder::new("mock_addr".to_owned(), &msg)
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

        let complete = IbcHooksBuilder::new("mock_addr".to_owned(), &msg)
            .callback_contract(Addr::unchecked("callback_addr"))
            .build()
            .unwrap();
        let value: serde_json::Value = serde_json::from_str(&complete).unwrap();
        let expected_value = json!({
            "wasm": {
                "contract": "mock_addr",
                "msg": {"withdraw": {}},
            },
            "ibc_callback": "callback_addr"
        });
        assert_eq!(value, expected_value);
    }
}
