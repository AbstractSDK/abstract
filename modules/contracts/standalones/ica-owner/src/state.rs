use cosmwasm_std::{Addr, Storage};
use cw_ica_controller::{ibc::types::metadata::TxEncoding, types::state::ChannelState};
use cw_ica_controller::ibc::types::packet::acknowledgement::Data;
use cw_ica_controller::types::query_msg::IcaQueryResult;
use cw_storage_plus::{Item, Map};
use crate::MyStandaloneError;

#[cosmwasm_schema::cw_serde]
pub struct Config {
    /// The code ID of the cw-ica-controller contract.
    pub ica_controller_code_id: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
/// The item used to store the count of the cw-ica-controller contracts.
pub const ICA_COUNT: Item<u64> = Item::new("ica_count");
/// The map used to store the state of the cw-ica-controller contracts.
pub const ICA_STATES: Map<u64, IcaContractState> = Map::new("ica_states");
/// The item used to map contract addresses to ICA IDs.
pub const CONTRACT_ADDR_TO_ICA_ID: Map<Addr, u64> = Map::new("contract_addr_to_ica_id");

/// (ica_id) -> sequence number. `u64` is the type used in the
/// Cosmos SDK for sequence numbers:
///
/// <https://github.com/cosmos/ibc-go/blob/a25f0d421c32b3a2b7e8168c9f030849797ff2e8/modules/core/02-client/keeper/keeper.go#L116-L125>
const SEQUENCE_NUMBER: Map<u64, u64> = Map::new("sn");

/// (ica_id, sequence_number) -> sender
///
/// Maps packets to the address that sent them.
const PENDING: Map<(u64, u64), Addr> = Map::new("pending");

/// (ica_id, sequence) -> execute data
pub const EXECUTE_RECEIPTS: Map<(u64, u64), Data> = Map::new("executes");
pub const QUERY_RECEIPTS: Map<(u64, u64), IcaQueryResult> = Map::new("queries");


/// Increments and returns the next sequence number.
pub(crate) fn increment_sequence_number(
    storage: &mut dyn Storage,
    ica_id: u64,
) -> Result<u64, MyStandaloneError> {
    let seq = SEQUENCE_NUMBER
        .may_load(storage, ica_id.clone())?
        .unwrap_or_default()
        .checked_add(1)
        .ok_or(MyStandaloneError::SequenceOverflow)?;
    SEQUENCE_NUMBER.save(storage, ica_id, &seq)?;
    Ok(seq)
}


/// IcaContractState is the state of the cw-ica-controller contract.
#[cosmwasm_schema::cw_serde]
pub struct IcaContractState {
    pub contract_addr: Addr,
    pub ica_state: Option<IcaState>,
}

/// IcaState is the state of the ICA.
#[cosmwasm_schema::cw_serde]
pub struct IcaState {
    pub ica_id: u64,
    pub ica_addr: String,
    pub tx_encoding: TxEncoding,
    pub channel_state: ChannelState,
}

impl IcaContractState {
    /// Creates a new [`IcaContractState`].
    pub fn new(contract_addr: Addr) -> Self {
        Self {
            contract_addr,
            ica_state: None,
        }
    }
}

impl IcaState {
    /// Creates a new [`IcaState`].
    pub fn new(
        ica_id: u64,
        ica_addr: String,
        tx_encoding: TxEncoding,
        channel_state: ChannelState,
    ) -> Self {
        Self {
            ica_id,
            ica_addr,
            tx_encoding,
            channel_state,
        }
    }
}
