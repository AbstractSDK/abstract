//! # Endpoints
//! This module provides endpoints for a base contract.
//! Each endpoint is a trait that can be implemented by a base contract to support a specific endpoint.
//!
//! > *If you're not familiar with the concept of endpoints/entry points, please read the [CosmWasm documentation](https://book.cosmwasm.com/basics/entry-points.html).*
//!
//! Endpoints are similar to CosmWasm's entry points but are more opinionated and standardized.
//! We'll go over all the available endpoints and their functionality and use-cases. But first, let's go over the message format expected by an Abstract module.
//!
//! ## Message format
//! Each Abstract module accepts a fixed message format that can be customized by the developer to add their own functionality.
//!
//! The base massage format is defined [here](abstract_core::base) as follows:
//! ```rust
//! use abstract_ica::IbcResponseMsg;
//! use cosmwasm_std::Empty;
//!
//! /// EndpointMsg to the Base.
//! #[cosmwasm_schema::cw_serde]
//! pub enum ExecuteMsg<BaseMsg, ModuleMsg, ReceiveMsg = Empty> {
//!     /// A base configuration message.
//!     Base(BaseMsg),
//!     /// An app request.
//!     App(ModuleMsg),
//!     /// IbcReceive to process IBC callbacks
//!     IbcCallback(IbcResponseMsg),
//!     /// Receive endpoint for CW20 / external service integrations
//!     Receive(ReceiveMsg),
//! }
//!
//! #[cosmwasm_schema::cw_serde]
//! pub struct InstantiateMsg<BaseMsg, ModuleMsg = Empty> {
//!     /// base instantiate msg
//!     pub base: BaseMsg,
//!     /// custom instantiate msg
//!     pub app: ModuleMsg,
//! }
//!
//! #[cosmwasm_schema::cw_serde]
//! pub enum QueryMsg<BaseMsg, ModuleMsg = Empty> {
//!     /// A query message to the base.
//!     Base(BaseMsg),
//!     /// Custom query
//!     App(ModuleMsg),
//! }
//!
//! #[cosmwasm_schema::cw_serde]
//! pub struct MigrateMsg<BaseMsg = Empty, ModuleMsg = Empty> {
//!     /// base migrate msg
//!     pub base: BaseMsg,
//!     /// custom migrate msg
//!     pub app: ModuleMsg,
//! }
//!
//! ```
//! Every `Base` variant or field is implemented by the base contract such as the [App](https://crates.io/crates/abstract-app), [API](https://crates.io/crates/abstract-api) and [IBC-host](https://crates.io/crates/abstract-ibc-host) contracts.
//! These contracts then expose a type that requires the missing `App` variant types to be provided. The rust type system
//! is then smart enough to accept the correct message type for each custom endpoint.
//!
//! Lets have a look at the available endpoints.
//!
//! ## Execute
//! The execute endpoint is the most common endpoint. A base contract implements it to handle its `Base` variant messages and forwards `App` or `Receive` variant messages to a custom execute handler.
//! Here's the implementation for the App contract:
//!
//!
//! ```rust,ignore
//! use abstract_sdk::core::app::{ExecuteMsg, AppExecuteMsg};
//! use abstract_app::{AppContract, AppError};
//! # use abstract_sdk::base::ExecuteEndpoint;
//! # use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
//! # use schemars::JsonSchema;
//! # use serde::Serialize;
//!
//! impl <Error: From<cosmwasm_std::StdError> + From<AppError> + 'static, CustomExecMsg: Serialize + JsonSchema + AppExecuteMsg, CustomInitMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg: Serialize + JsonSchema >
//! ExecuteEndpoint for AppContract <Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg > {
//!     
//!     // Expected entrypoint ExecuteMsg type, imported from abstract_core.
//!     // As you can see from the type definition, the `AppContract` accepts a custom `AppExecuteMsg`
//!     // type that is inserted into the expected execute message.
//!     type ExecuteMsg = ExecuteMsg<CustomExecMsg, ReceiveMsg>;
//!     
//!     fn execute(
//!         self,
//!         deps: DepsMut,
//!         env: Env,
//!         info: MessageInfo,
//!         msg: Self::ExecuteMsg,
//!     ) -> Result<Response, Error> {
//!         // match message on base message format with help of the type system
//!         match msg {
//!             ExecuteMsg::Base(exec_msg) => self
//!                 .base_execute(deps, env, info, exec_msg)
//!                 .map_err(From::from),
//!             // handle the other messages with a custom handler set by the developer
//!             // by passing `self` to the handlers we expose all the features and APIs that the base contract provides through the SDK.
//!             ExecuteMsg::App(request) => self.execute_handler()?(deps, env, info, self, request),
//!             ExecuteMsg::IbcCallback(msg) => self.ibc_callback(deps, env, info, msg),
//!             ExecuteMsg::Receive(msg) => self.receive(deps, env, info, msg),
//!            
//!         }
//!     }
//! }
//! ```
//! Two variants reside in the ExecuteMsg enum:
//!
//! #### Receive
//! The receive endpoint is used to handle messages sent from external contracts, most commonly the [CW20](https://crates.io/crates/cw20) contract.
//!
//! #### IbcCallback
//! The IbcCallback endpoint is used to handle IBC responses that indicate that a certain IBC action has been completed.
//!
//!
//! ## Instantiate
//! The instantiate endpoint is used to initialize a base contract. It has a field for a custom `App` message that is passed to the instantiate handler.
//!
//! ## Query
//! The query endpoint is used to query a contract. It is similar to the execute endpoint but it also forwards custom `App` variant queries.
//!
//! ## Migrate
//! Same as the instantiate endpoint but for migrating a contract.
//!
//! ## Reply
//! The reply endpoint is used to handle internal replies. Each reply handler is matched with a reply-id. Both are supplied to the contract builder.

mod execute;
mod ibc_callback;
mod instantiate;
pub(crate) mod migrate;
mod query;
mod receive;
mod reply;

// Provide endpoints under ::base::traits::
pub use execute::ExecuteEndpoint;
pub use ibc_callback::IbcCallbackEndpoint;
pub use instantiate::InstantiateEndpoint;
pub use migrate::MigrateEndpoint;
pub use query::QueryEndpoint;
pub use receive::ReceiveEndpoint;
pub use reply::ReplyEndpoint;
