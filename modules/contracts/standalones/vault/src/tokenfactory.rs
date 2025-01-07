//! All of the tokenfactory-related code. Currently it relies on osmosis version
//!
//! It's done in separate module to make it easier switch to different tokenfactory version or cw20

use cosmwasm_std::{Binary, CosmosMsg, Empty, Env};
use osmosis_std::types::osmosis::tokenfactory::v1beta1::MsgCreateDenom;

use crate::SHARE_SUBDENOM;

pub fn share_denom(env: &Env) -> String {
    format!("factory/{}/{}", env.contract.address, SHARE_SUBDENOM)
}

#[allow(deprecated)]
pub fn create_denom(env: &Env) -> CosmosMsg<Empty> {
    CosmosMsg::Stargate {
        type_url: MsgCreateDenom::TYPE_URL.to_owned(),
        value: Binary::new(
            MsgCreateDenom {
                sender: env.contract.address.to_string(),
                subdenom: SHARE_SUBDENOM.to_owned(),
            }
            .to_proto_bytes(),
        ),
    }
}
