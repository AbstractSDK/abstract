use abstract_app::better_sdk::bank::TransferInterface;
use abstract_app::better_sdk::sdk::{AbstractAppBase, ContractInfo};
use abstract_app::better_sdk::{
    contexts::{AppExecCtx, AppInstantiateCtx, AppMigrateCtx, AppQueryCtx},
    execution::Execution,
};
use abstract_app::AppError;
use cosmwasm_std::{coins, ensure, Addr, BankMsg, CosmosMsg, ReplyOn, StdError};
use cw_storage_plus::Item;
use sylvia::{contract, entry_points};

use abstract_app::better_sdk::execution_stack::{CustomEvents, ExecutionStack};

pub struct SylviaContract<'a> {
    counter: Item<'a, u64>,
}

impl Default for SylviaContract<'_> {
    fn default() -> Self {
        Self {
            counter: Item::new("counter"),
        }
    }
}

#[entry_points]
#[contract]
#[contract_type(abstract_app::better_sdk::sdk::AbstractApp)]
#[error(AppError)]
#[messages(abstract_app::better_sdk::sdk as Base)]
impl SylviaContract<'_> {
    pub fn new() -> Self {
        Self::default()
    }

    #[msg(instantiate)]
    pub fn instantiate(
        &self,
        ctx: &mut AppInstantiateCtx,
        couter_init: u64,
    ) -> Result<(), AppError> {
        self.counter.save(ctx.deps.storage, &couter_init)?;

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

        Ok(())
    }

    #[msg(migrate)]
    pub fn migrate(&self, ctx: &mut AppMigrateCtx, new_counter: u64) -> Result<(), AppError> {
        self.counter.save(ctx.deps.storage, &new_counter)?;
        Ok(())
    }

    #[msg(exec)]
    pub fn increment<'a>(
        &self,
        mut ctx: AppExecCtx<'a>,
        i: u64,
    ) -> Result<AppExecCtx<'a>, AppError> {
        let counter = self.counter.load(ctx.deps.storage)?;
        ensure!(counter < 10, StdError::generic_err("Limit reached"));
        ensure!(i < 10, StdError::generic_err("Unauthorized"));

        self.counter.save(ctx.deps.storage, &(counter + 1))?;

        let amount = coins(145, "ujuno");

        ctx.bank()
            .with_reply(ReplyOn::Always, 76)
            .transfer(amount.clone(), &Addr::unchecked("adair".to_string()))?;

        ctx.add_attribute("action", "execute_test");

        Ok(ctx)
    }

    #[msg(query)]
    pub fn count(&self, ctx: AppQueryCtx) -> Result<u64, AppError> {
        Ok(self.counter.load(ctx.deps.storage)?)
    }
}

impl AbstractAppBase for SylviaContract<'_> {
    type Error = AppError;
    const INFO: ContractInfo = ("COUNTER_ID", "v1.0.1", Some("Metadata"));
    const DEPENDENCIES: &'static [abstract_core::objects::dependency::StaticDependency] = &[];
}

pub mod ibc_callbacks {
    use abstract_app::better_sdk::contexts::AppExecCtx;
    use abstract_sdk::AbstractSdkError;
    use cosmwasm_std::{Response, StdError};
    use sylvia::interface;

    #[interface]
    pub trait IbcCallback {
        type Error: From<StdError> + From<AbstractSdkError>;

        #[msg(exec)]
        fn dex_callback(
            &self,
            ctx: AppExecCtx,
            callback_msg: Option<cosmwasm_std::Binary>,
            result: polytone::callbacks::Callback,
        ) -> Result<Response, AbstractSdkError> {
            Ok(Response::new())
        }

        #[msg(exec)]
        fn random_callback(
            &self,
            ctx: AppExecCtx,
            callback_msg: Option<cosmwasm_std::Binary>,
            result: polytone::callbacks::Callback,
        ) -> Result<Response, AbstractSdkError> {
            Ok(Response::new())
        }
    }
}

fn main() {
    test::main();
}

pub mod test {

    use crate::entry_points::{execute, instantiate, query};
    use crate::sv::{
        ContractExecMsg, ContractQueryMsg, ExecMsg, ImplInstantiateMsg, InstantiateMsg, QueryMsg,
    };

    use abstract_app::better_sdk::sdk::sv::AbstractAppBaseExecMsg;
    use abstract_core::app::BaseInstantiateMsg;
    use abstract_testing::addresses::{
        TEST_ANS_HOST, TEST_MANAGER, TEST_MODULE_FACTORY, TEST_VERSION_CONTROL,
    };
    use abstract_testing::mock_querier;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_json, Attribute};

    pub fn main() {
        let mut deps = mock_dependencies();
        deps.querier = mock_querier();
        let response = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info(TEST_MODULE_FACTORY, &[]),
            InstantiateMsg {
                base: BaseInstantiateMsg {
                    ans_host_address: TEST_ANS_HOST.to_string(),
                    version_control_address: TEST_VERSION_CONTROL.to_string(),
                },
                module: ImplInstantiateMsg { couter_init: 8 },
            },
        )
        .unwrap();

        assert_eq!(response.messages.len(), 5);
        assert_eq!(response.events.len(), 1);

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info(TEST_MANAGER, &[]),
            ContractExecMsg::Base(AbstractAppBaseExecMsg::UpdateConfig {
                ans_host_address: None,
                version_control_address: None,
            }),
        )
        .unwrap();

        let response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info(TEST_MODULE_FACTORY, &[]),
            ContractExecMsg::SylviaContract(ExecMsg::Increment { i: 7 }),
        )
        .unwrap();

        assert_eq!(response.messages.len(), 1);
        assert_eq!(
            response.attributes,
            vec![Attribute::new("action", "execute_test")]
        );

        let response = query(
            deps.as_ref(),
            mock_env(),
            ContractQueryMsg::SylviaContract(QueryMsg::Count {}),
        )
        .unwrap();

        let count: i32 = from_json(response).unwrap();
        assert_eq!(count, 9i32);
    }
}
