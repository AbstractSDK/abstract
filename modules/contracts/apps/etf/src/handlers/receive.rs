use cosmwasm_std::{from_json, DepsMut, Env, MessageInfo};
use cw20::Cw20ReceiveMsg;
use cw_asset::{Asset, AssetInfo};

use crate::{
    contract::{EtfApp, EtfResult},
    error::EtfError,
    handlers::execute,
    msg::Cw20HookMsg,
    state::{State, STATE},
};

// TODO: Custom handler
/// handler function invoked when the vault dapp contract receives
/// a transaction. In this case it is triggered when either a LP tokens received
/// by the contract or when the deposit asset is a cw20 asset.
pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    module: EtfApp,
    cw20_msg: Cw20ReceiveMsg,
) -> EtfResult {
    match from_json(&cw20_msg.msg)? {
        Cw20HookMsg::Claim {} => {
            let state: State = STATE.load(deps.storage)?;
            if msg_info.sender != state.share_token_address {
                return Err(EtfError::NotLPToken {
                    token: msg_info.sender.to_string(),
                });
            }
            let sender = deps.as_ref().api.addr_validate(&cw20_msg.sender)?;
            execute::try_withdraw_liquidity(deps, env, module, sender, cw20_msg.amount)
        }
        Cw20HookMsg::Deposit {} => {
            // Construct deposit asset
            let asset = Asset {
                info: AssetInfo::Cw20(msg_info.sender.clone()),
                amount: cw20_msg.amount,
            };
            execute::try_provide_liquidity(deps, msg_info, module, asset, Some(cw20_msg.sender))
        }
    }
}
