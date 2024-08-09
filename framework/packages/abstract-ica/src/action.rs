use cosmwasm_std::{Binary, Coin, CosmosMsg};

/// Interchain Account Action
#[cosmwasm_schema::cw_serde]
#[non_exhaustive]
pub enum IcaAction {
    // Execute on the ICA
    Execute(IcaExecute),
    // Query on the ICA
    // Query(IcaQuery),
    // Send funds to the ICA
    Fund {
        funds: Vec<Coin>,
        // Optional receiver address
        // Should be formatted in expected formatting
        // EVM: HexBinary
        // Cosmos: Addr
        receiver: Option<Binary>,
        memo: Option<String>,
    },
    // ... other actions?
}

impl IcaAction {
    // Used to set ordering
    pub fn discriminant(&self) -> u8 {
        match self {
            IcaAction::Execute(_) => 0,
            IcaAction::Fund { .. } => 1,
            // IcaAction::Query(_) => 2,
        }
    }
}

/// Queries first
/// Execute second
/// Funds transfers last
impl PartialOrd for IcaAction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IcaAction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Eq for IcaAction {}

#[cosmwasm_schema::cw_serde]
#[non_exhaustive]
pub enum IcaExecute {
    Evm {
        // TODO: move types to `abstract-ica` crate
        msgs: Vec<polytone_evm::evm::EvmMsg<String>>,
        callback: Option<polytone_evm::callbacks::CallbackRequest>,
    },
    // Cosmos {
    //     msgs: Vec<CosmosMsg>,
    //     callback: Option<CallbackRequest>,
    // },
}

// pub enum IcaQuery {
// 	Evm {
// 		// encoded data
// 		// ...
// 	},
// 	Cosmos {
// 	    // Encoded data
// 		// ...
// 	}
// }

#[cosmwasm_schema::cw_serde]
pub struct IcaActionResponse {
    /// messages that call the underlying implementations (be it polytone/cw-ica-controller/etc)
    pub msgs: Vec<CosmosMsg>,
}
