use alloy::primitives::{Address, AddressError};
use alloy_sol_types::SolValue;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::HexBinary;

// TODO: this is duplicated from `evm-note!`

/// Marks either `String` or valid EVM `Address`.
///
/// String is used in unverified types, such as messages and query responses.
/// Addr is used in verified types, which are to be stored in blockchain state.
///
/// This trait is intended to be used as a generic in type definitions.
pub trait EvmAddressLike {}

impl EvmAddressLike for String {}

impl EvmAddressLike for HexBinary {}

impl EvmAddressLike for Address {}

/// A message to be sent to the EVM
#[cw_serde]
pub enum EvmMsg<T: EvmAddressLike> {
    /// Call a contract with the given data
    /// @param to The address of the contract to call, must pass checksum validation
    /// @param data The data to send to the contract
    Call { to: T, data: HexBinary },
}

impl<T> EvmMsg<T>
where
    T: EvmAddressLike,
{
    pub fn call(to: T, data: impl Into<HexBinary>) -> Self {
        EvmMsg::Call {
            to,
            data: data.into(),
        }
    }
}

impl EvmMsg<String> {
    /// Check the validity of the addresses
    pub fn check(self) -> Result<EvmMsg<Address>, AddressError> {
        match self {
            EvmMsg::Call { to, data } => {
                let to: Address = Address::parse_checksummed(&to, None)?;
                Ok(EvmMsg::Call { to, data })
            }
        }
    }
}

impl EvmMsg<Address> {
    pub fn encode(self) -> Vec<u8> {
        match self {
            EvmMsg::Call { to, data } => {
                let data = data.to_vec();
                let call = abi_types::CallMessage {
                    to,
                    data: data.to_vec().into(),
                }
                .abi_encode();
                abi_types::EvmMsg {
                    msgType: abi_types::EvmMsgType::Call,
                    message: call.into(),
                }
                .abi_encode()
            }
        }
    }

    #[allow(dead_code)]
    fn unchecked(self) -> EvmMsg<String> {
        match self {
            EvmMsg::Call { to, data } => EvmMsg::Call {
                to: to.to_string(),
                data,
            },
        }
    }
}

pub mod abi_types {
    use alloy::sol_types::sol;

    // Copied directly from solidity/src/Requests.sol
    sol! {
      // Reflect the CW Packet
    struct Packet {
        string sender;
        Msg msg;
    }

    // Message called on the voice
    struct Msg {
        MsgType msgType;
        bytes[] data;
    }

    // Type of voice message
    enum MsgType {
        Execute
    //    Query
    }

    // Message called on the proxy contract
    // Reflect EvmMsg<Address>
    struct EvmMsg {
        EvmMsgType msgType;
        bytes message;
    }

    // Type of message called on proxy contract
    enum EvmMsgType {
        Call,
    }

    // Data to execute by proxy
    struct CallMessage {
        address to;
        bytes data;
    }

    struct ExecuteResult {
        bool success;
        bytes data;
    }

    struct ExecuteResponsePacket {
        address executedBy;
        ExecuteResult[] result;
    }
    struct Token {
        address denom;
        uint128 amount;
    }

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod call_message {
        use super::*;

        #[test]
        fn call_message() {
            let msg = abi_types::CallMessage {
                to: "0x785B548D3d7064F77A26e479AC7847DBCE0c1B46"
                    .parse()
                    .unwrap(),
                data: HexBinary::from_hex("b49004e9").unwrap().to_vec().into(),
            };

            let encoded = msg.abi_encode();
            let actual = HexBinary::from(encoded);

            let expected = HexBinary::from_hex("0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000785b548d3d7064f77a26e479ac7847dbce0c1b4600000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000004b49004e900000000000000000000000000000000000000000000000000000000").unwrap();

            assert_eq!(actual, expected);
        }

        #[test]
        fn evm_msg() {
            let msg = EvmMsg::Call {
                to: "0x785B548D3d7064F77A26e479AC7847DBCE0c1B46".to_string(),
                data: HexBinary::from_hex("b49004e9").unwrap(),
            }
            .check()
            .unwrap();

            let encoded = msg.encode();
            let actual = HexBinary::from(encoded);

            let expected = HexBinary::from_hex("00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000785b548d3d7064f77a26e479ac7847dbce0c1b4600000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000004b49004e900000000000000000000000000000000000000000000000000000000").unwrap();

            assert_eq!(actual, expected);
        }
    }
}
