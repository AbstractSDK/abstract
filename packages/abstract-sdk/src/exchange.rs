use abstract_os::{dex::OfferAsset, objects::AssetEntry, EXCHANGE};
use cosmwasm_std::{CosmosMsg, Deps, StdResult};

use crate::Dependency;
use abstract_os::dex::RequestMsg;

/// Perform actions on an exchange API.
/// WIP
pub trait Exchange: Dependency {
    fn swap(
        &self,
        deps: Deps,
        dex: String,
        offer_asset: OfferAsset,
        ask_asset: AssetEntry,
    ) -> StdResult<CosmosMsg> {
        self.call_api_dependency(
            deps,
            EXCHANGE,
            &RequestMsg::Swap {
                offer_asset,
                ask_asset,
                dex: Some(dex),
                max_spread: None,
                belief_price: None,
            },
            vec![],
        )
    }
}
