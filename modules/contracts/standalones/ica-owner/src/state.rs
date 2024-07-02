use cosmwasm_std::Addr;
use cw_ica_controller::{ibc::types::metadata::TxEncoding, types::state::ChannelState};
use cw_storage_plus::{Item, Map};

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
