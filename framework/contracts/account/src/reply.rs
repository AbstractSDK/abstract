use crate::{
    contract::{AccountResponse, AccountResult},
    modules::INSTALL_MODULES_CONTEXT,
};
use abstract_std::{
    account::state::ADMIN_CALL_TO_CONTEXT,
    objects::{
        module::{assert_module_data_validity, Module},
        module_reference::ModuleReference,
    },
};
use cosmwasm_std::{DepsMut, Reply, Response, StdError};

/// Add the message's data to the response
pub fn forward_response_reply(result: Reply) -> AccountResult {
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
pub fn admin_action_reply(deps: DepsMut) -> AccountResult {
    ADMIN_CALL_TO_CONTEXT.remove(deps.storage);

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
