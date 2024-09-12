use crate::{
    contract::{AccountResponse, AccountResult},
    modules::INSTALL_MODULES_CONTEXT,
};
use abstract_std::objects::{
    module::{assert_module_data_validity, Module},
    module_reference::ModuleReference,
};
use cosmwasm_std::{DepsMut, Reply, Response, StdError};

/// Add the message's data to the response
pub fn forward_response_data(result: Reply) -> AccountResult {
    // get the result from the reply
    let res = result.result.into_result().map_err(StdError::generic_err)?;

    // log and add data if needed
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
