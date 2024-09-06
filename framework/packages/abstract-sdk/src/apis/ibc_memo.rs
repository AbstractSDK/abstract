mod hooks;
mod pfm;

pub use hooks::HookMemoBuilder;
pub use pfm::PfmMemoBuilder;

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::Addr;
    use serde_json::json;

    #[test]
    fn memo_middleware() {
        let minimal = PfmMemoBuilder::new("channel-1")
            .build("foo")
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

        let complete = PfmMemoBuilder::new("channel-1")
            .port("different_port")
            .timeout("10m")
            .retries(4)
            .hop("channel-2")
            .build("foo")
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

        let multimultihop = PfmMemoBuilder::new("channel-1")
            .hop("channel-2")
            .hop("channel-3")
            .build("receiver")
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

        let minimal = HookMemoBuilder::new("mock_addr".to_owned(), &msg)
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

        let complete = HookMemoBuilder::new("mock_addr".to_owned(), &msg)
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
