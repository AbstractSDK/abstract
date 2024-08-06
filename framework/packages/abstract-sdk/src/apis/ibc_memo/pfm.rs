use std::collections::BTreeMap;

use cosmwasm_std::Addr;
use serde_cw_value::Value;

use super::IbcMemoBuilder;

/// Builder for [Packet Forward Middleware](https://github.com/cosmos/ibc-apps/tree/main/middleware/packet-forward-middleware) memos.
pub struct PacketForwardMiddlewareBuilder {
    channel: String,
    receiver: Option<Addr>,
    port: Option<String>,
    timeout: Option<String>,
    retries: Option<u8>,
    next: Option<BTreeMap<Value, Value>>,
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
        self.next = Some(next_memo.build_value_map());
        self
    }
}

impl IbcMemoBuilder for PacketForwardMiddlewareBuilder {
    fn build_value_map(self) -> BTreeMap<Value, Value> {
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
            forward_value.insert(Value::String("next".to_owned()), Value::Map(next));
        }

        BTreeMap::from([(
            Value::String("forward".to_owned()),
            Value::Map(forward_value.into_iter().collect()),
        )])
    }
}
