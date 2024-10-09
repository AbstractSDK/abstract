use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use schemars::JsonSchema;
use serde::Serialize;

use crate::base::handler::Handler;

/// Trait for a contract's Execute entry point.
pub trait ExecuteEndpoint: Handler {
    /// The message type for the Execute entry point.
    type ExecuteMsg: Serialize + JsonSchema;

    /// Handler for the Execute endpoint.
    fn execute(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::ExecuteMsg,
    ) -> Result<Response, Self::Error>;
}

/// Trait for a custom contract Execute entry point
///
/// To use it:
/// - create a custom execute enum, with your custom methods
/// - copy all desired endpoints of App [`abstract_std::app::ExecuteMsg`] or Adapter [`abstract_std::adapter::ExecuteMsg`]
/// - Add your custom type to the end of export_endpoints!() and cw_orch_interface!() macros
pub trait CustomExecuteHandler<Module: Handler>: Sized {
    /// Module execute message (`crate::msg::ExecuteMsg` of your module)
    type ExecuteMsg;

    // Can't use try_into because of conflicting impls, in core
    /// Convert custom execute message to your module execute message, or if not possible return custom
    fn try_into_base(self) -> Result<Self::ExecuteMsg, Self>;

    /// Handle of custom execute methods of the module.
    ///
    /// This method is used if [`CustomExecuteHandler::into_execute_msg`] returned Error
    fn custom_execute(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        module: Module,
    ) -> Result<Response, Module::Error>;
}
