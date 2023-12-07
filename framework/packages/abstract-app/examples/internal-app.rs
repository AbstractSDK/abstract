use abstract_app::better_sdk::bank::TransferInterface;
use abstract_app::better_sdk::execution::Execution;
use abstract_app::better_sdk::execution_stack::CustomEvents;
use abstract_app::better_sdk::execution_stack::DepsAccess;
use abstract_app::better_sdk::execution_stack::ExecutionStack;
use abstract_app::export_endpoints;
use abstract_app::{mock::MockError, AppContract};
use abstract_core::app::BaseInstantiateMsg;
use abstract_core::base::InstantiateMsg as CoreInstantiateMsg;
use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::testing::mock_info;
use cosmwasm_std::DepsMut;
use cosmwasm_std::Env;
use cosmwasm_std::MessageInfo;
use cosmwasm_std::{coins, Empty};

use cosmwasm_std::{Addr, BankMsg, CosmosMsg, ReplyOn};

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub counter: u64,
}
// We try a mock app,
pub type InternalDepsApp<'a, T> = AppContract<
    'a,
    T,
    // MockModule,
    MockError,
    InstantiateMsg,
    Empty,
    Empty,
    Empty,
    Empty,
    Empty,
>;

fn mock_app<'a, T: DepsAccess>(deps: T) -> InternalDepsApp<'a, T> {
    InternalDepsApp::new(deps, "internal_shit", "this version is cool", None).with_instantiate(
        |app, msg| {
            let amount = coins(145, "ujuno");

            // Those messages are proxy routed messages with account action execution
            app.bank()
                .transfer(amount.clone(), &Addr::unchecked("robin".to_string()))?;

            app.bank()
                .with_reply(ReplyOn::Always, 76)
                .transfer(amount.clone(), &Addr::unchecked("adair".to_string()))?;

            // This is very similar execept we are batching the execution here
            app.executor().execute(vec![
                CosmosMsg::Bank(BankMsg::Send {
                    to_address: "nicolas_through_proxy".to_string(),
                    amount: amount.clone(),
                }),
                CosmosMsg::Bank(BankMsg::Send {
                    to_address: "nicolas_through_proxy".to_string(),
                    amount: amount.clone(),
                }),
            ])?;

            app.executor().execute_with_reply(
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
            app.push_app_message(CosmosMsg::Bank(BankMsg::Send {
                to_address: "nicolas".to_string(),
                amount,
            }));
            app.add_event("abstract_execution", vec![("action", "test-event-value")]);

            Ok(())
        },
    )
}

export_endpoints!(mock_app, InternalDepsApp<'b>, 'b);

fn main() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);
    let response = instantiate(
        deps.as_mut(),
        env,
        info,
        CoreInstantiateMsg {
            module: InstantiateMsg { counter: 0 },
            base: BaseInstantiateMsg {
                ans_host_address: "ans".to_string(),
                version_control_address: "version-control".to_string(),
            },
        },
    )
    .unwrap();
}
