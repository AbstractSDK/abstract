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

#[cosmwasm_schema::cw_serde]
#[non_exhaustive]
pub enum IcaExecute {
    Evm {
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
