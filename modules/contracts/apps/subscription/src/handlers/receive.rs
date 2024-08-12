use cosmwasm_std::{from_json, DepsMut, Env, MessageInfo};
use cw20::Cw20ReceiveMsg;
use cw_asset::{Asset, AssetInfo};

use crate::{
    contract::{SubscriptionApp, SubscriptionResult},
    handlers::execute,
    msg::DepositHookMsg,
};

// TODO: custom execute msg
pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    module: SubscriptionApp,
    cw20_msg: Cw20ReceiveMsg,
) -> SubscriptionResult {
    match from_json(cw20_msg.msg)? {
        DepositHookMsg::Pay { subscriber_addr } => {
            // Construct deposit asset
            let asset = Asset {
                info: AssetInfo::Cw20(msg_info.sender.clone()),
                amount: cw20_msg.amount,
            };
            let subscriber_addr = deps
                .api
                .addr_validate(&subscriber_addr.unwrap_or(cw20_msg.sender))?;
            execute::try_pay(module, deps, env, asset, subscriber_addr)
        }
    }
}
