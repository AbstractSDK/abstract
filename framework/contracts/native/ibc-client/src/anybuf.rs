#![allow(unused)]

use anybuf::Anybuf;

pub struct Coin {
    pub denom: String,  // 1
    pub amount: String, // 2
}

impl Coin {
    pub fn to_anybuf(&self) -> Anybuf {
        Anybuf::new()
            .append_string(1, &self.denom)
            .append_string(2, &self.amount)
    }
}

impl From<cosmwasm_std::Coin> for Coin {
    fn from(coin: cosmwasm_std::Coin) -> Self {
        Self {
            denom: coin.denom,
            amount: coin.amount.to_string(),
        }
    }
}

pub mod ibc {
    use super::*;

    pub struct Height {
        /// the revision that the client is currently on
        pub revision_number: u64, // 1
        /// the height within the given revision
        pub revision_height: u64, // 2
    }

    impl Height {
        pub fn to_anybuf(&self) -> Anybuf {
            Anybuf::new()
                .append_uint64(1, self.revision_number)
                .append_uint64(2, self.revision_height)
        }
    }

    /// Ibc transfer
    pub struct MsgTransfer {
        /// the port on which the packet will be sent
        pub source_port: String, // 1
        /// the channel by which the packet will be sent
        pub source_channel: String, // 2
        /// the tokens to be transferred
        pub token: Option<Coin>, // 3
        /// the sender address
        pub sender: String, // 4
        /// the recipient address on the destination chain
        pub receiver: String, // 5
        /// Timeout height relative to the current block height.
        /// The timeout is disabled when set to 0.
        pub timeout_height: Option<Height>, // 6
        /// Timeout timestamp in absolute nanoseconds since unix epoch.
        /// The timeout is disabled when set to 0.
        pub timeout_timestamp: u64, // 7
        /// optional memo
        pub memo: String, // 8
    }

    impl MsgTransfer {
        pub fn type_url() -> String {
            "/ibc.applications.transfer.v1.MsgTransfer".to_owned()
        }

        pub fn to_anybuf(&self) -> Anybuf {
            let token = self.token.as_ref().map(Coin::to_anybuf).unwrap_or_default();
            let timeout_height = self
                .timeout_height
                .as_ref()
                .map(Height::to_anybuf)
                .unwrap_or_default();
            Anybuf::new()
                .append_string(1, &self.source_port)
                .append_string(2, &self.source_channel)
                .append_message(3, &token)
                .append_string(4, &self.sender)
                .append_string(5, &self.receiver)
                .append_message(6, &timeout_height)
                .append_uint64(7, self.timeout_timestamp)
                .append_string(8, &self.memo)
        }
    }

    pub struct MsgTransferResponse {
        pub sequence: u64, // 1
    }

    impl MsgTransferResponse {
        pub fn decode(data: &cosmwasm_std::Binary) -> Result<Self, anybuf::BufanyError> {
            let bufany = anybuf::Bufany::deserialize(data.as_ref())?;
            let sequence = bufany
                .uint64(1)
                .ok_or(anybuf::BufanyError::UnexpectedEndOfData)?;
            Ok(Self { sequence })
        }
    }
}

#[cfg(test)]
mod test {

    use ibc_proto::{cosmos::base::v1beta1::Coin, ibc::applications::transfer::v1::MsgTransfer};
    use prost::{Message, Name};

    #[coverage_helper::test]
    fn test_outcomes() {
        let source_port = "123".to_owned();
        let source_channel = "321".to_owned();
        let token = Some(Coin {
            denom: "denom".to_owned(),
            amount: "456".to_owned(),
        });
        let sender = "sender".to_owned();
        let receiver = "receiver".to_owned();
        let timeout_height = Some(ibc_proto::ibc::core::client::v1::Height {
            revision_number: 45,
            revision_height: 345,
        });
        let timeout_timestamp = 56234;
        let memo = "some_memo".to_owned();

        let expected_bytes = MsgTransfer {
            source_port: source_port.clone(),
            source_channel: source_channel.clone(),
            token: token.clone(),
            sender: sender.clone(),
            receiver: receiver.clone(),
            timeout_height,
            timeout_timestamp,
            memo: memo.clone(),
        }
        .encode_to_vec();
        let anybuf_out = super::ibc::MsgTransfer {
            source_port,
            source_channel,
            token: token.map(|c| super::Coin {
                denom: c.denom,
                amount: c.amount,
            }),
            sender,
            receiver,
            timeout_height: timeout_height.map(|h| super::ibc::Height {
                revision_number: h.revision_number,
                revision_height: h.revision_height,
            }),
            timeout_timestamp,
            memo,
        }
        .to_anybuf()
        .into_vec();
        assert_eq!(anybuf_out, expected_bytes);
        assert_eq!(MsgTransfer::type_url(), super::ibc::MsgTransfer::type_url());
    }
}
