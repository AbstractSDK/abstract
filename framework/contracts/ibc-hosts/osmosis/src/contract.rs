use crate::error::OsmoError;
use abstract_core::ibc_host::ExecuteMsg;
use abstract_ibc_host::chains::OSMOSIS;
use abstract_ibc_host::Host;
use abstract_macros::abstract_response;
use abstract_sdk::{
    base::{ExecuteEndpoint, InstantiateEndpoint, MigrateEndpoint, QueryEndpoint, ReplyEndpoint},
    core::{
        abstract_ica::StdAck,
        dex::DexAction,
        ibc_host::{InstantiateMsg, MigrateMsg, QueryMsg},
        OSMOSIS_HOST,
    },
    Execution,
};
use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Empty, Env, IbcPacketReceiveMsg, IbcReceiveResponse,
    MessageInfo, Reply, ReplyOn, Response,
};
use dex::host_exchange::Osmosis;
use dex::LocalDex;

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[abstract_response(OSMOSIS_HOST)]
pub(crate) struct OsmosisHostResponse;

pub type OsmoHost = Host<OsmoError, Empty, DexAction>;
pub type OsmoResult<T = Response> = Result<T, OsmoError>;
pub type IbcOsmoResult = Result<IbcReceiveResponse, OsmoError>;

const OSMO_HOST: OsmoHost = OsmoHost::new(OSMOSIS_HOST, CONTRACT_VERSION, OSMOSIS, None);

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(deps: DepsMut, env: Env, info: MessageInfo, msg: InstantiateMsg) -> OsmoResult {
    OSMO_HOST.instantiate(deps, env, info, msg)?;
    Ok(OsmosisHostResponse::action("instantiate"))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<DexAction>,
) -> OsmoResult {
    // will only process base requests as there is no exec handler set.
    OSMO_HOST.execute(deps, env, info, msg)
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
    let acknowledgement = StdAck::fail(format!("action {action:?} failed"));

    // execute and expect reply after execution
    let (msgs, reply_id) = host.resolve_dex_action(deps.as_ref(), action, &exchange)?;
    let proxy_msg =
        host.executor(deps.as_ref())
            .execute_with_reply(msgs, ReplyOn::Success, reply_id)?;

    Ok(IbcReceiveResponse::new()
        .set_ack(acknowledgement)
        .add_submessage(proxy_msg)
        .add_attribute("action", "handle_app_action"))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> OsmoResult {
    OSMO_HOST.reply(deps, env, reply)
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> OsmoResult<Binary> {
    OSMO_HOST.query(deps, env, msg)
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> OsmoResult {
    OSMO_HOST.migrate(deps, env, msg)
}
