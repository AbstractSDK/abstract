use abstract_ibc_host::chains::OSMOSIS;
use abstract_ibc_host::Host;
use abstract_ibc_host::HostError;
use abstract_os::ibc_host::{BaseInstantiateMsg, MigrateMsg, QueryMsg};
use abstract_os::{dex::RequestMsg, OSMOSIS_HOST};

use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, IbcPacketReceiveMsg, IbcReceiveResponse, MessageInfo,
    Response, StdResult,
};
use cw2::{get_contract_version, set_contract_version};
use semver::Version;

use crate::error::OsmoError;
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type OsmoHost<'a> = Host<'a, RequestMsg>;
pub type OsmoResult = Result<Response, OsmoError>;
const OSMO_HOST: OsmoHost<'static> = OsmoHost::new(&[]);

// Supported exchanges on XXX
// ...

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: BaseInstantiateMsg,
) -> OsmoResult {
    OsmoHost::instantiate(
        deps,
        env,
        info,
        msg,
        OSMOSIS_HOST,
        CONTRACT_VERSION,
        OSMOSIS,
    )?;
    Ok(Response::default())
}

/// we look for a the proper reflect contract to relay to and send the message
/// We cannot return any meaningful response value as we do not know the response value
/// of execution. We just return ok if we dispatched, error if we failed to dispatch
#[entry_point]
pub fn ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, HostError> {
    OSMO_HOST.handle_packet(deps, env, msg, handle_packet)
}

fn handle_packet(
    _deps: DepsMut,
    _env: Env,
    _caller_channel: String,
    _host: OsmoHost,
    _packet: RequestMsg,
) -> Result<IbcReceiveResponse, HostError> {
    todo!()
    // match packet {
    //     RequestMsg::ProvideLiquidity {
    //         assets,
    //         dex,
    //         max_spread,
    //     } => {
    //         todo!()
    //         // let dex_name = dex.unwrap();
    //         // if assets.len() < 2 {
    //         //     return Err(DexError::TooFewAssets {});
    //         // }
    //         // provide_liquidity(deps.as_ref(), env, info, api, assets, dex_name, max_spread)
    //     }
    //     RequestMsg::ProvideLiquiditySymmetric {
    //         offer_asset,
    //         paired_assets,
    //         dex,
    //     } => {
    //         todo!()
    //         // let dex_name = dex.unwrap();
    //         // if paired_assets.is_empty() {
    //         //     return Err(DexError::TooFewAssets {});
    //         // }
    //         // provide_liquidity_symmetric(
    //         //     deps.as_ref(),
    //         //     env,
    //         //     info,
    //         //     api,
    //         //     offer_asset,
    //         //     paired_assets,
    //         //     dex_name,
    //         // )
    //     }
    //     RequestMsg::WithdrawLiquidity {
    //         lp_token,
    //         amount,
    //         dex,
    //     } => {
    //         todo!()
    //         // let dex_name = dex.unwrap();
    //         // withdraw_liquidity(deps.as_ref(), env, info, api, (lp_token, amount), dex_name)
    //     }

    //     RequestMsg::Swap {
    //         offer_asset,
    //         ask_asset,
    //         dex,
    //         max_spread,
    //         belief_price,
    //     } => {
    //         todo!()
    //         // add default dex in future (osmosis??)
    //         // let dex_name = dex.unwrap();
    //         // swap(
    //         //     deps.as_ref(),
    //         //     env,
    //         //     info,
    //         //     api,
    //         //     offer_asset,
    //         //     ask_asset,
    //         //     dex_name,
    //         //     max_spread,
    //         //     belief_price,
    //         // )
    //     }
    // }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, OsmoError> {
    OSMO_HOST.handle_query(deps, env, msg, None)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    let storage_version: Version = get_contract_version(deps.storage)?.version.parse().unwrap();
    if storage_version < version {
        set_contract_version(deps.storage, OSMOSIS_HOST, CONTRACT_VERSION)?;
    }
    Ok(Response::default())
}
