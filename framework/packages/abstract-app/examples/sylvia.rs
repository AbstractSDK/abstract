use abstract_app::better_sdk::bank::TransferInterface;
use abstract_app::better_sdk::execute::AppExecCtx;
use abstract_app::better_sdk::execution::Execution;
use abstract_app::better_sdk::instantiate::AppInstantiateCtx;
use abstract_app::better_sdk::migrate::AppMigrateCtx;
use abstract_sdk::AbstractSdkError;
use cosmwasm_std::{ensure, StdError, coins, Addr, ReplyOn, CosmosMsg, BankMsg};
use cw_storage_plus::Item;
use sylvia::{contract, entry_points};
use sylvia::cw_std::{StdResult, Response};

use abstract_app::better_sdk::execution_stack::{DepsAccess, CustomEvents, ExecutionStack};
use abstract_app::better_sdk::implementations::AbstractApp;

pub struct SylviaContract<'a>{
    counter: Item<'a, u64>
}


impl Default for SylviaContract<'_>{
    fn default() -> Self {
        Self {
            counter: Item::new("counter")
        }
    }
}


#[entry_points]
#[contract]
#[contract_type(AbstractApp)]
#[error(AbstractSdkError)]
impl SylviaContract<'_> {

    pub fn new() -> Self {
        Self::default()
    }

    #[msg(instantiate)]
    pub fn instantiate<'a>(&self, mut ctx: AppInstantiateCtx<'a>, admin: String) -> abstract_sdk::AbstractSdkResult<AppInstantiateCtx<'a> > {
        let admin_addr = ctx.api().addr_validate(&admin)?;
        ctx.base_state.admin.set(ctx.deps.branch(), Some(admin_addr))?;

        let amount = coins(145, "ujuno");

        // Those messages are proxy routed messages with account action execution
        ctx.bank()
            .transfer(amount.clone(), &Addr::unchecked("robin".to_string()))?;


        ctx.bank()
            .with_reply(ReplyOn::Always, 76)
            .transfer(amount.clone(), &Addr::unchecked("adair".to_string()))?;



        // This is very similar execept we are batching the execution here
        ctx.executor().execute(vec![
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "nicolas_through_proxy".to_string(),
                amount: amount.clone(),
            }),
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "nicolas_through_proxy".to_string(),
                amount: amount.clone(),
            }),
        ])?;

        ctx.executor().execute_with_reply(
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
        ctx.push_app_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: "nicolas".to_string(),
            amount,
        }));
        ctx.add_event("abstract_execution", vec![("action", "test-event-value")]);


        Ok(ctx)
    }

    #[msg(migrate)]
    pub fn migrate<'a>(&self, mut ctx: AppMigrateCtx<'a>, admin: String) -> StdResult<AppMigrateCtx<'a>> {
        let admin_addr = ctx.api().addr_validate(&admin)?;
        ctx.base_state.admin.set(ctx.deps.branch(), Some(admin_addr))?;
        Ok(ctx)
    }

    #[msg(exec)]
    pub fn increment(&self, ctx: AppExecCtx) -> StdResult<Response> {
        let counter = self.counter.load(ctx.deps.storage)?;

        ensure!(counter < 10, StdError::generic_err("Limit reached"));

        self.counter.save(ctx.deps.storage, &(counter + 1))?;
        Ok(Response::new())
    }
}


fn main(){
    test::main();
}


pub mod test{

    use abstract_core::app::BaseInstantiateMsg;
    use abstract_testing::addresses::{TEST_ANS_HOST, TEST_VERSION_CONTROL, TEST_MODULE_FACTORY};
    use abstract_testing::mock_querier;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use crate::sv::{ImplInstantiateMsg, InstantiateMsg};
    use crate::entry_points::instantiate;
    pub fn main(){
        
        let mut deps = mock_dependencies();
        deps.querier = mock_querier();
        let response = instantiate(deps.as_mut(), mock_env(), mock_info(TEST_MODULE_FACTORY, &[]), InstantiateMsg{
            base: BaseInstantiateMsg {
                ans_host_address: TEST_ANS_HOST.to_string(),
                version_control_address: TEST_VERSION_CONTROL.to_string(),
            },
            module: ImplInstantiateMsg{
                admin: "abstract".to_string()
            }
        }).unwrap();

        assert_eq!(response.messages.len(), 5);
        assert_eq!(response.events.len(), 1);
    }
}

// /// This is the message that the app expects
// #[cw_serde]
// pub struct AppInstantiateMsg{
//     admin: String
// }

// /// This is the message that the contract expects
// #[cw_serde]
// pub struct InstantiateMsg{
//     base: BaseInstantiateMsg,
//     module: AppInstantiateMsg
// }


// pub fn instantiate(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     msg: InstantiateMsg
// ) -> Result<
//     sylvia::cw_std::Response<sylvia::cw_std::Empty>,
//     sylvia::cw_std::StdError,
// > {
//     msg.dispatch(&SylviaContract::new(), (deps, env, info)).map_err(Into::into)
// }


// impl InstantiateMsg {
//     pub fn dispatch(
//         self,
//         contract: &SylviaContract<'_>,
//         ctx: (
//             sylvia::cw_std::DepsMut<sylvia::cw_std::Empty>,
//             sylvia::cw_std::Env,
//             sylvia::cw_std::MessageInfo,
//         ),
//     ) -> StdResult<Response> {
//         let Self { base, module } = self;

//         let AppInstantiateMsg{ admin } = module;

//         contract.instantiate(TryInto::try_into((ctx, base))?, admin).map_err(Into::into)
//     }
// }