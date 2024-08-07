
use cosmwasm_std::{Coin, Coins, CosmosMsg};
use evm_note::polytone::ack::Callback;
use polytone::callbacks::CallbackRequest;

use crate::evm::EvmMsg;

/// Interchain Account Action
#[cosmwasm_schema::cw_serde]
pub enum IcaAction {
	// Execute on the ICA
	Execute(IcaExecute),
	// Query on the ICA
	// Query(IcaQuery),
	// Send funds to the ICA
	Fund(Vec<Coin>),
	// ... other actions?
}

/// Queries first
/// Execute second
/// Funds transfers last
impl PartialOrd for IcaAction {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		match (self, other) {
			(IcaAction::Execute(_), IcaAction::Execute(_)) => Some(std::cmp::Ordering::Equal),
			(IcaAction::Fund(_), IcaAction::Fund(_)) => Some(std::cmp::Ordering::Equal),
			(IcaAction::Execute(_), IcaAction::Fund(_)) => Some(std::cmp::Ordering::Greater),
			(IcaAction::Fund(_), IcaAction::Execute(_)) => Some(std::cmp::Ordering::Less),
			// (IcaAction::Query(_), IcaAction::Query(_)) => Some(std::cmp::Ordering::Equal),
			// (IcaAction::Query(_), _) => Some(std::cmp::Ordering::Less),
			// (_, IcaAction::Query(_)) => Some(std::cmp::Ordering::Greater),
		}
	}
}

#[cosmwasm_schema::cw_serde]
pub enum IcaExecute{
	Evm {
        msgs: Vec<EvmMsg<String>>,
        callback: Option<CallbackRequest>
	},
	Cosmos {
        msgs: Vec<CosmosMsg>,
        callback: Option<CallbackRequest>
	}
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
	pub msgs: Vec<CosmosMsg>
}