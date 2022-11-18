use cosmwasm_std::from_binary;
use cosmwasm_std::DepsMut;
use cosmwasm_std::{Env, MessageInfo};
use cw20::Cw20ReceiveMsg;
use cw_asset::Asset;
use cw_asset::AssetInfo;

use abstract_os::etf::state::{State, STATE};
use abstract_os::etf::DepositHookMsg;

use crate::contract::{EtfApp, EtfResult};
use crate::error::EtfError;
use crate::handlers::execute;

/// handler function invoked when the vault dapp contract receives
/// a transaction. In this case it is triggered when either a LP tokens received
/// by the contract or when the deposit asset is a cw20 asset.
pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    dapp: EtfApp,
    cw20_msg: Cw20ReceiveMsg,
) -> EtfResult {
    match from_binary(&cw20_msg.msg)? {
        DepositHookMsg::WithdrawLiquidity {} => {
            let state: State = STATE.load(deps.storage)?;
            if msg_info.sender != state.liquidity_token_addr {
                return Err(EtfError::NotLPToken {
                    token: msg_info.sender.to_string(),
                });
            }
            execute::try_withdraw_liquidity(deps, env, dapp, cw20_msg.sender, cw20_msg.amount)
        }
        DepositHookMsg::ProvideLiquidity {} => {
            // Construct deposit asset
            let asset = Asset {
                info: AssetInfo::Cw20(msg_info.sender.clone()),
                amount: cw20_msg.amount,
            };
            execute::try_provide_liquidity(deps, msg_info, dapp, asset, Some(cw20_msg.sender))
        }
    }
}
