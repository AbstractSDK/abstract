use abstract_ibc_host::chains::OSMOSIS;
use abstract_ibc_host::Host;

use abstract_os::abstract_ica::StdAck;
use abstract_os::dex::DexAction;
use abstract_os::ibc_host::{InstantiateMsg, MigrateMsg, QueryMsg};
use abstract_os::OSMOSIS_HOST;

use abstract_sdk::{InstantiateEndpoint, QueryEndpoint, ReplyEndpoint};
use cosmwasm_std::Reply;
use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, IbcPacketReceiveMsg, IbcReceiveResponse, MessageInfo,
    Response, StdResult,
};
use cw2::{get_contract_version, set_contract_version};

use dex::host_exchange::Osmosis;
use dex::LocalDex;
use semver::Version;

use crate::error::OsmoError;
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type OsmoHost = Host<OsmoError, DexAction>;
pub type OsmoResult = Result<Response, OsmoError>;
pub type IbcOsmoResult = Result<IbcReceiveResponse, OsmoError>;

const OSMO_HOST: OsmoHost = OsmoHost::new(OSMOSIS_HOST, CONTRACT_VERSION, OSMOSIS);

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(deps: DepsMut, env: Env, info: MessageInfo, msg: InstantiateMsg) -> OsmoResult {
    OSMO_HOST.instantiate(deps, env, info, msg)?;
    Ok(Response::default())
}

/// we look for a the proper reflect contract to relay to and send the message
/// We cannot return any meaningful response value as we do not know the response value
/// of execution. We just return ok if we dispatched, error if we failed to dispatch
#[entry_point]
pub fn ibc_packet_receive(deps: DepsMut, env: Env, msg: IbcPacketReceiveMsg) -> IbcOsmoResult {
    OSMO_HOST.handle_packet(deps, env, msg, handle_app_action)
}

fn handle_app_action(deps: DepsMut, _env: Env, host: OsmoHost, packet: DexAction) -> IbcOsmoResult {
    let exchange = Osmosis {
        local_proxy_addr: host.proxy_address.clone(),
    };
    let action = packet;
    let acknowledgement = StdAck::fail(format!("action {:?} failed", action));

    // execute and expect reply after execution
    let proxy_msg = host.resolve_dex_action(deps, action, &exchange, true)?;
    Ok(IbcReceiveResponse::new()
        .set_ack(acknowledgement)
        .add_submessage(proxy_msg)
        .add_attribute("action", "handle_app_action"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> OsmoResult {
    OSMO_HOST.reply(deps, env, reply)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    OSMO_HOST.query(deps, env, msg)
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
