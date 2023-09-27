use crate::handlers::execute;
use cosmwasm_std::{from_binary, DepsMut, Env, MessageInfo};
use cw20::Cw20ReceiveMsg;
use cw_asset::{Asset, AssetInfo};

use crate::{
    contract::{SubscriptionApp, SubscriptionResult},
    msg::DepositHookMsg,
};

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    app: SubscriptionApp,
    cw20_msg: Cw20ReceiveMsg,
) -> SubscriptionResult {
    match from_binary(&cw20_msg.msg)? {
        DepositHookMsg::Pay { os_id } => {
            // Construct deposit asset
            let asset = Asset {
                info: AssetInfo::Cw20(msg_info.sender.clone()),
                amount: cw20_msg.amount,
            };
            execute::try_pay(app, deps, env, msg_info, asset, os_id)
        }
    }
}
