use abstract_core::proto::ibc::ProtoMsgTransfer;
use cosmrs::{tx::Msg, ErrorReport, Result};

/// MsgSend represents a message to send coins from one account to another.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct MsgTransfer {
    /// Sender's address.
    pub source_port: String,
    pub source_channel: String,
    pub token: Option<cosmrs::Coin>,
    pub sender: cosmrs::AccountId,
    pub receiver: cosmrs::AccountId,
    pub timeout_height: Option<cosmrs::tendermint::block::Height>,
    pub timeout_revision: Option<u64>,
    pub timeout_timestamp: u64,
    pub memo: Option<String>,
}

impl Msg for MsgTransfer {
    type Proto = ProtoMsgTransfer;
}

impl TryFrom<ProtoMsgTransfer> for MsgTransfer {
    type Error = ErrorReport;

    fn try_from(proto: ProtoMsgTransfer) -> Result<MsgTransfer> {
        MsgTransfer::try_from(&proto)
    }
}

impl TryFrom<&ProtoMsgTransfer> for MsgTransfer {
    type Error = ErrorReport;

    fn try_from(proto: &ProtoMsgTransfer) -> Result<MsgTransfer> {
        Ok(MsgTransfer {
            source_port: proto.source_port.parse()?,
            source_channel: proto.source_channel.parse()?,
            token: proto.token.clone().map(TryFrom::try_from).transpose()?,
            sender: proto.sender.parse()?,
            receiver: proto.receiver.parse()?,
            timeout_height: proto
                .timeout_height
                .clone()
                .map(|h| h.revision_height.try_into())
                .transpose()?,
            timeout_revision: proto.timeout_height.clone().map(|h| h.revision_number),
            timeout_timestamp: proto.timeout_timestamp,
            memo: proto.memo.clone(),
        })
    }
}

impl From<MsgTransfer> for ProtoMsgTransfer {
    fn from(coin: MsgTransfer) -> ProtoMsgTransfer {
        ProtoMsgTransfer::from(&coin)
    }
}

impl From<&MsgTransfer> for ProtoMsgTransfer {
    fn from(msg: &MsgTransfer) -> ProtoMsgTransfer {
        ProtoMsgTransfer {
            source_port: msg.source_port.clone(),
            source_channel: msg.source_channel.clone(),
            token: msg.token.clone().map(Into::into),
            sender: msg.sender.to_string(),
            receiver: msg.receiver.to_string(),
            timeout_height: msg.timeout_height.map(|h| {
                cosmrs::proto::ibc::core::client::v1::Height {
                    revision_number: msg.timeout_revision.unwrap(),
                    revision_height: h.value(),
                }
            }),
            timeout_timestamp: msg.timeout_timestamp,
            memo: msg.memo.clone(),
        }
    }
}
