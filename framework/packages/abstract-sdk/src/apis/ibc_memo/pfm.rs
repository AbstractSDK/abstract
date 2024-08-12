use std::collections::BTreeMap;

use serde_cw_value::Value;

use super::IbcMemoBuilder;

/// Builder for [Packet Forward Middleware](https://github.com/cosmos/ibc-apps/tree/main/middleware/packet-forward-middleware) memos.
pub struct PacketForwardMiddlewareBuilder {
    receiver: String,
    port: Option<String>,
    hops: Vec<PacketForwardMiddlewareHop>,
}

impl PacketForwardMiddlewareBuilder {
    /// Forward memo builder
    pub fn new(receiver: impl Into<String>) -> Self {
        Self {
            receiver: receiver.into(),
            port: None,
            hops: vec![],
        }
    }

    /// Port, defaults to "transfer"
    pub fn port(mut self, port: impl Into<String>) -> Self {
        self.port = Some(port.into());
        self
    }

    /// Channel hop
    pub fn hop(mut self, channel: impl Into<String>) -> Self {
        self.hops.push(PacketForwardMiddlewareHop::new(channel));
        self
    }

    /// Hop modifier (applies only on last added hop):
    /// Timeout duration, for example: "10m"
    pub fn timeout(mut self, timeout: impl Into<String>) -> Self {
        if let Some(last_hop) = self.hops.last_mut() {
            last_hop.timeout = Some(timeout.into());
        }
        self
    }

    /// Hop modifier (applies only on last added hop):
    /// Retries number
    pub fn retries(mut self, retries: u8) -> Self {
        if let Some(last_hop) = self.hops.last_mut() {
            last_hop.retries = Some(retries);
        }
        self
    }
}

struct PacketForwardMiddlewareHop {
    channel: String,
    timeout: Option<String>,
    retries: Option<u8>,
}

impl PacketForwardMiddlewareHop {
    pub fn new(channel: impl Into<String>) -> Self {
        Self {
            channel: channel.into(),
            timeout: None,
            retries: None,
        }
    }
}

impl IbcMemoBuilder for PacketForwardMiddlewareBuilder {
    fn build_value_map(self) -> BTreeMap<Value, Value> {
        let PacketForwardMiddlewareBuilder {
            receiver,
            port,
            hops,
        } = self;
        let port = port.unwrap_or("transfer".to_owned());

        let mut forwards = hops
            .into_iter()
            .map(|hop| ForwardMemo {
                receiver: None,
                port: port.clone(),
                channel: hop.channel,
                timeout: hop.timeout,
                retries: hop.retries,
            })
            .collect::<Vec<_>>();
        // Destination have to know receiver
        if let Some(last_hop) = forwards.last_mut() {
            last_hop.receiver = Some(receiver);
        }

        // Building message from behind because it's easier to satisfy borrow checker this way
        let mut head = BTreeMap::new();
        for forward in forwards.into_iter().rev() {
            let mut forward_msg = forward.build_value_map();
            if !head.is_empty() {
                let next = head;
                forward_msg.insert(Value::String("next".to_owned()), Value::Map(next));
            }
            head = BTreeMap::from([(Value::String("forward".to_owned()), Value::Map(forward_msg))]);
        }
        head
    }
}

// Forward structure
struct ForwardMemo {
    receiver: Option<String>,
    port: String,
    channel: String,
    timeout: Option<String>,
    retries: Option<u8>,
}

impl IbcMemoBuilder for ForwardMemo {
    fn build_value_map(self) -> BTreeMap<Value, Value> {
        let receiver = self.receiver.unwrap_or("pfm".to_owned());
        let mut forward_value = BTreeMap::from([
            (
                Value::String("receiver".to_owned()),
                Value::String(receiver),
            ),
            (Value::String("port".to_owned()), Value::String(self.port)),
            (
                Value::String("channel".to_owned()),
                Value::String(self.channel),
            ),
        ]);
        if let Some(timeout) = self.timeout {
            forward_value.insert(Value::String("timeout".to_owned()), Value::String(timeout));
        }
        if let Some(retries) = self.retries {
            forward_value.insert(Value::String("retries".to_owned()), Value::U8(retries));
        }
        forward_value
    }
}
