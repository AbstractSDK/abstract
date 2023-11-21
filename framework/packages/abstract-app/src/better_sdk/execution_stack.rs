use abstract_core::proxy::ExecuteMsg;
use abstract_sdk::{AbstractSdkResult, AccountAction};
use cosmwasm_std::{
    wasm_execute, Api, Attribute, Binary, CosmosMsg, Deps, DepsMut, Event, ReplyOn, Response,
    SubMsg,
};

use super::account_identification::AccountIdentification;

pub trait DepsAccess {
    fn deps_mut<'a: 'b, 'b>(&'a mut self) -> DepsMut<'b>;
    fn deps<'a: 'b, 'b>(&'a self) -> Deps<'b>;

    fn api<'a: 'b, 'b>(&'a self) -> &'b dyn Api {
        self.deps().api
    }
}

#[derive(Clone)]
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
                        let msg: CosmosMsg = wasm_execute(
                            proxy_addr.clone(),
                            &ExecuteMsg::ModuleAction { msgs: a.messages() },
                            vec![],
                        )?
                        .into();
                        SubMsg::new(msg)
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
pub trait CustomEvents {
    fn add_event(&mut self, event_name: &str, attributes: Vec<(&str, &str)>);
    fn events(&self) -> Vec<Event>;

    fn add_attributes(&mut self, attributes: Vec<(&str, &str)>);
    fn add_attribute(&mut self, key: &str, value: &str) {
        self.add_attributes(vec![(key, value)])
    }
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

// #[cfg(test)]
// mod test {
//     use super::*;
//     use abstract_testing::prelude::*;
//     use speculoos::prelude::*;

//     struct MockBinding;

//     impl AccountIdentification for MockBinding {
//         fn proxy_address(&self, _deps: Deps) -> AbstractSdkResult<Addr> {
//             Ok(Addr::unchecked(TEST_PROXY))
//         }
//     }

//     mod account {
//         use super::*;
//         use cosmwasm_std::testing::mock_dependencies;

//         #[test]
//         fn test_proxy_address() {
//             let binding = MockBinding;
//             let deps = mock_dependencies();

//             let res = binding.proxy_address(deps.as_ref());
//             assert_that!(res)
//                 .is_ok()
//                 .is_equal_to(Addr::unchecked(TEST_PROXY));
//         }

//         #[test]
//         fn test_manager_address() {
//             let binding = MockBinding;
//             let mut deps = mock_dependencies();

//             deps.querier = MockQuerierBuilder::default()
//                 .with_contract_item(TEST_PROXY, MANAGER, &Some(Addr::unchecked(TEST_MANAGER)))
//                 .build();

//             assert_that!(binding.manager_address(deps.as_ref()))
//                 .is_ok()
//                 .is_equal_to(Addr::unchecked(TEST_MANAGER));
//         }

//         #[test]
//         fn test_account() {
//             let mut deps = mock_dependencies();
//             deps.querier = MockQuerierBuilder::default()
//                 .with_contract_item(TEST_PROXY, MANAGER, &Some(Addr::unchecked(TEST_MANAGER)))
//                 .build();

//             let expected_account_base = AccountBase {
//                 manager: Addr::unchecked(TEST_MANAGER),
//                 proxy: Addr::unchecked(TEST_PROXY),
//             };

//             let binding = MockBinding;
//             assert_that!(binding.account_base(deps.as_ref()))
//                 .is_ok()
//                 .is_equal_to(expected_account_base);
//         }

//         #[test]
//         fn account_id() {
//             let mut deps = mock_dependencies();
//             deps.querier = MockQuerierBuilder::default()
//                 .with_contract_item(TEST_PROXY, ACCOUNT_ID, &TEST_ACCOUNT_ID)
//                 .build();

//             let binding = MockBinding;
//             assert_that!(binding.account_id(deps.as_ref()))
//                 .is_ok()
//                 .is_equal_to(TEST_ACCOUNT_ID);
//         }
//     }
// }
