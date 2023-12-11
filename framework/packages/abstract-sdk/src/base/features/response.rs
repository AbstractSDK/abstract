use abstract_core::proxy::ExecuteMsg;
use cosmwasm_std::{
    wasm_execute, Attribute, Binary, CosmosMsg, DepsMut, Env, Event, MessageInfo, ReplyOn,
    Response, SubMsg,
};

use crate::{AbstractSdkResult, AccountAction};

use super::AccountIdentification;

#[derive(Clone, PartialEq, Debug)]
pub enum Executable {
    CosmosMsg(CosmosMsg),
    SubMsg {
        msgs: Vec<CosmosMsg>,
        reply_on: ReplyOn,
        id: u64,
    },
    AccountAction(AccountAction),
}
/// A list of messages that can be executed
/// Can only be appended to and iterated over.
pub struct Executables(pub(crate) Vec<Executable>);

impl Default for Executables {
    fn default() -> Self {
        Self(Vec::with_capacity(8))
    }
}

impl Executables {
    pub fn push(&mut self, msg: Executable) {
        self.0.push(msg)
    }
}

pub trait ExecutionStack: Sized + AccountIdentification {
    fn stack_mut(&mut self) -> &mut Executables;
    /// Push an executable to the stack
    fn push_executable(&mut self, executable: Executable) {
        self.stack_mut().push(executable);
    }
    /// Get the manager address for the current account.
    fn push_app_message(&mut self, msg: CosmosMsg) {
        self.stack_mut().push(Executable::CosmosMsg(msg));
    }
    /// Get the manager address for the current account.
    fn push_app_messages(&mut self, msgs: Vec<CosmosMsg>) {
        self.stack_mut()
            .0
            .extend(msgs.iter().map(|msg| Executable::CosmosMsg(msg.clone())));
    }
    /// Get the manager address for the current account.
    fn push_proxy_message(&mut self, msg: CosmosMsg) {
        self.push_proxy_messages(vec![msg])
    }
    /// Get the manager address for the current account.
    fn push_proxy_messages(&mut self, msgs: Vec<CosmosMsg>) {
        self.stack_mut()
            .push(Executable::AccountAction(AccountAction::from_vec(msgs)));
    }
    /// NEVER USE INSIDE YOUR CONTRACTS
    /// Only used for unwrapping the messages to the Response inside abstract
    fn _unwrap_for_response(&mut self) -> AbstractSdkResult<Vec<SubMsg>> {
        let proxy_addr = self.proxy_address()?.to_string();

        let stack = self.stack_mut();

        stack
            .0
            .iter()
            .map(|e| {
                let msg = match e {
                    Executable::AccountAction(a) => {
                        let options = a.options();
                        match options.reply {
                            Some(reply_options) => {
                                let msg = if reply_options.with_data {
                                    wasm_execute(
                                        proxy_addr.clone(),
                                        &ExecuteMsg::ModuleActionWithData {
                                            msg: a.messages()[0].clone(),
                                        },
                                        vec![],
                                    )?
                                    .into()
                                } else {
                                    wasm_execute(
                                        proxy_addr.clone(),
                                        &ExecuteMsg::ModuleAction { msgs: a.messages() },
                                        vec![],
                                    )?
                                    .into()
                                };
                                SubMsg {
                                    id: reply_options.id,
                                    msg,
                                    gas_limit: None,
                                    reply_on: reply_options.reply_on,
                                }
                            }
                            None => {
                                let msg: CosmosMsg = wasm_execute(
                                    proxy_addr.clone(),
                                    &ExecuteMsg::ModuleAction { msgs: a.messages() },
                                    vec![],
                                )?
                                .into();
                                SubMsg::new(msg)
                            }
                        }
                    }
                    Executable::CosmosMsg(msg) => SubMsg::new(msg.clone()),
                    Executable::SubMsg { msgs, reply_on, id } => {
                        let msg: CosmosMsg = wasm_execute(
                            proxy_addr.clone(),
                            &ExecuteMsg::ModuleAction { msgs: msgs.clone() },
                            vec![],
                        )?
                        .into();
                        SubMsg {
                            id: *id,
                            msg: msg.clone(),
                            gas_limit: None,
                            reply_on: reply_on.clone(),
                        }
                    }
                };
                Ok(msg)
            })
            .collect()
    }
}

/// Allows to register events for the response corresponding to the current endpoint
pub trait CustomEvents {
    /// Adds events to the response generated after the current endpoint
    fn add_event<A: Into<Attribute>>(
        &mut self,
        event_name: &str,
        attributes: impl IntoIterator<Item = A>,
    );
    /// Events getter
    fn events(&self) -> Vec<Event>;

    /// Adds attributes to the response generated after the current endpoint
    fn add_attributes<A: Into<Attribute>>(&mut self, attributes: impl IntoIterator<Item = A>);
    /// Adds one attribute to the response generated after the current endpoint
    fn add_attribute(&mut self, key: &str, value: &str) {
        self.add_attributes(vec![(key, value)])
    }
    /// Attributes getter
    fn attributes(&self) -> Vec<Attribute>;
}
pub trait CustomData {
    fn data(&self) -> Option<Binary>;
    fn set_data(&mut self, data: impl Into<Binary>);
}
pub trait ResponseGenerator: ExecutionStack + CustomEvents + CustomData {
    fn _generate_response(&mut self) -> AbstractSdkResult<Response> {
        let resp = Response::new()
            .add_events(self.events())
            .add_attributes(self.attributes())
            .add_submessages(self._unwrap_for_response()?);
        Ok(if let Some(data) = self.data() {
            resp.set_data(data)
        } else {
            resp
        })
    }
}

impl<T> ResponseGenerator for T where T: ExecutionStack + CustomEvents + CustomData {}

/// Allows to detect which environment is currently being executed on
pub trait HasExecutableEnv {}

impl HasExecutableEnv for (DepsMut<'_>, Env, MessageInfo) {}

impl HasExecutableEnv for (DepsMut<'_>, Env) {}
