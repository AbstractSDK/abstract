use crate::{
    contract::{AccountResponse, AccountResult},
    modules::INSTALL_MODULES_CONTEXT,
    msg::{ExecuteMsg, ICS20_CALLBACKS},
};
use abstract_std::{
    account::{state::CALLING_TO_AS_ADMIN, ICS20PacketIdentifier},
    objects::{
        module::{assert_module_data_validity, Module},
        module_reference::ModuleReference,
    },
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{from_json, DepsMut, Reply, Response, StdError};

/// Add the message's data to the response
pub(crate) fn forward_response_reply(result: Reply) -> AccountResult {
    let res = result.result.into_result().map_err(StdError::generic_err)?;

    #[allow(deprecated)]
    let resp = if let Some(data) = res.data {
        AccountResponse::new(
            "forward_response_data_reply",
            vec![("response_data", "true")],
        )
        .set_data(data)
    } else {
        AccountResponse::new(
            "forward_response_data_reply",
            vec![("response_data", "false")],
        )
    };
    Ok(resp)
}

/// Remove the storage for an admin call after execution
pub(crate) fn admin_action_reply(deps: DepsMut) -> AccountResult {
    CALLING_TO_AS_ADMIN.remove(deps.storage);

    Ok(Response::new())
}

/// Adds the modules dependencies
pub(crate) fn register_dependencies(deps: DepsMut) -> AccountResult {
    let modules = INSTALL_MODULES_CONTEXT.load(deps.storage)?;

    for (module, module_addr) in &modules {
        assert_module_data_validity(&deps.querier, module, module_addr.clone())?;

        match module {
            Module {
                reference: ModuleReference::App(_),
                info,
            }
            | Module {
                reference: ModuleReference::Adapter(_),
                info,
            } => {
                let id = info.id();
                // assert version requirements
                let dependencies =
                    crate::versioning::assert_install_requirements(deps.as_ref(), &id)?;
                crate::versioning::set_as_dependent(deps.storage, id, dependencies)?;
            }
            Module {
                reference: ModuleReference::Standalone(_),
                info,
            } => {
                let id = info.id();
                // assert version requirements
                let dependencies =
                    crate::versioning::assert_install_requirements_standalone(deps.as_ref(), &id)?;
                crate::versioning::set_as_dependent(deps.storage, id, dependencies)?;
            }
            _ => (),
        };
    }

    Ok(Response::new())
}

use prost::Message;
#[derive(Clone, PartialEq, Message)]
struct MsgTransferResponse {
    #[prost(uint64, tag = "1")]
    pub sequence: u64,
}

pub fn register_sub_sequent_messages(deps: DepsMut, reply: Reply) -> AccountResult {
    let res = reply.result.into_result().map_err(StdError::generic_err)?;
    let transfer_response = MsgTransferResponse::decode(res.msg_responses[0].value.as_slice())?;

    let payload: TokenFlowPayload = from_json(reply.payload)?;

    // We register the callback for later use
    ICS20_CALLBACKS.save(
        deps.storage,
        ICS20PacketIdentifier {
            channel_id: payload.channel_id,
            sequence: transfer_response.sequence,
        },
        &payload.msgs,
    )?;

    Ok(Response::new())
}

#[cw_serde]
pub struct TokenFlowPayload {
    channel_id: String,
    msgs: Vec<ExecuteMsg>,
}
