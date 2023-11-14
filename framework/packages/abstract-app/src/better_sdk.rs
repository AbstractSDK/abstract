use self::bank::TransferInterface;
use self::execution::Execution;
use self::execution_stack::CustomEvents;
use self::execution_stack::DepsAccess;
use self::execution_stack::Executables;
use self::execution_stack::ExecutionStack;
use crate::AppError;
use abstract_sdk::AbstractSdkResult;
use abstract_sdk::base::AbstractContract;
use abstract_sdk::base::Handler;
use abstract_sdk::{namespaces::ADMIN_NAMESPACE, AbstractSdkError};
use cosmwasm_std::Event;
use cosmwasm_std::{
    coins, Addr, BankMsg, CosmosMsg, DepsMut, Empty, Env, MessageInfo, ReplyOn, Response,
};
use cw_controllers::Admin;
use cw_storage_plus::Item;
pub mod bank;
pub mod execution;
pub mod execution_stack;
mod implementations;
pub mod nameservice;
pub mod sdk;

// TODO: add macro here that generates the private struct below
// The macro should:
// 1. Generate a struct that contains this struct and the ModuleEnv
// 2. Generate a new function that instantiates the struct
// 3. Allow generation of endpoints simply (see sylvia)

// This is the custom struct defined by the dev.
// it contains all the state and handler functions of the contract.
pub struct TestContract<'a, Module: Handler + 'static, Error: From<AbstractSdkError> + 'static> {
    // Custom state goes here (like Sylvia)
    pub admin: Admin<'static>,
    pub config: Item<'static, u64>,

    // added automatically (by macro)
    pub deps: DepsMut<'a>,
    pub env: Env,
    pub info: MessageInfo,
    pub executable_stack: Executables,
    pub events: Vec<Event>,

    // Contract (added automatically as well)
    pub contract: AbstractContract<Module, Error>
}

// #[contract] TODO: re-enable this macro
impl<'a, Module: Handler + 'static, Error: From<AbstractSdkError> + 'static> TestContract<'a, Module, Error> {
    // new function must be implemented manually (like sylvia)
    pub fn new(deps: DepsMut<'a>, env: Env, info: MessageInfo, contract: AbstractContract<Module, Error>) -> Self {
        Self {
            admin: Admin::new(ADMIN_NAMESPACE),
            config: Item::new("cfg"),
            deps,
            env,
            info,
            executable_stack: Executables::default(),
            events: vec![],
            contract    
        }
    }

    // TODO: re-enable macro #[msg(instantiate)]
    // the macro removes the impl here and applies it to `_TestContract`
    pub fn instantiate(&mut self, admin: Option<String>) -> Result<(), AppError> {

        let admin = admin
            .map(|a| self.api().addr_validate(&a))
            .transpose()?;

        self.admin.set(self.deps.branch(), admin)?;
        self.config.save(self.deps.storage, &1u64)?;


        let amount = coins(145, "ujuno");

        // Those messages are proxy routed messages with account action execution
        self.bank()
            .transfer(amount.clone(), &Addr::unchecked("robin".to_string()))?;


        self.bank()
            .with_reply(ReplyOn::Always, 76)
            .transfer(amount.clone(), &Addr::unchecked("adair".to_string()))?;



        // This is very similar execept we are batching the execution here
        self.executor().execute(vec![
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "nicolas_through_proxy".to_string(),
                amount: amount.clone(),
            }),
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "nicolas_through_proxy".to_string(),
                amount: amount.clone(),
            }),
        ])?;

        self.executor().execute_with_reply(
            vec![
                CosmosMsg::Bank(BankMsg::Send {
                    to_address: "nicolas_through_proxy".to_string(),
                    amount: amount.clone(),
                }),
                CosmosMsg::Bank(BankMsg::Send {
                    to_address: "nicolas_through_proxy".to_string(),
                    amount: amount.clone(),
                }),
            ],
            cosmwasm_std::ReplyOn::Always,
            78,
        )?;

        // Those messages are messages sent by the contract directly
        self.push_app_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: "nicolas".to_string(),
            amount,
        }));
        self.add_event("abstract_execution", vec![("action", "test-event-value")]);

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::better_sdk::implementations::instantiate;

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn macros() {
        let mut deps = mock_dependencies();

        let resp = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("sender", &[]),
            Empty {},
        )
        .unwrap();

        assert_eq!(resp.messages.len(), 5);
        assert_eq!(resp.messages[0].reply_on, ReplyOn::Never);
        assert_eq!(resp.messages[1].reply_on, ReplyOn::Always);
        assert_eq!(resp.messages[1].id, 76);
    }
}
