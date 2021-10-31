use cosmwasm_std::{Addr, Coin, Decimal};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::ExecuteMsg as HandleMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnchorMsg {
    DepositStable {},
    RedeemStable {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BLunaMsg {
    Bond { validator: Addr },
    Unbond {},
    WithdrawUnbonded {},
}

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
