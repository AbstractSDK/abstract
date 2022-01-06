use cosmwasm_std::{Coin, Decimal};
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::ExecuteMsg as HandleMsg;

pub fn create_terraswap_msg(
    offer: Coin,
    belief_price: Decimal,
    max_spread: Option<Decimal>,
) -> HandleMsg {
    let offer = Asset {
        info: AssetInfo::NativeToken {
            denom: offer.denom.clone(),
        },
        amount: offer.amount,
    };
    HandleMsg::Swap {
        offer_asset: offer,
        belief_price: Some(belief_price),
        max_spread,
        to: None,
    }
}
