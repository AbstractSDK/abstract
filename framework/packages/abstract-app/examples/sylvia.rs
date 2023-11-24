use abstract_app::better_sdk::bank::TransferInterface;
use abstract_app::better_sdk::sdk::{AbstractAppBase, ModuleStateInfo};
use abstract_app::better_sdk::{
    contexts::{AppExecCtx, AppInstantiateCtx, AppMigrateCtx, AppQueryCtx},
    execution::Execution,
};
use abstract_app::AppError;
use cosmwasm_std::{coins, ensure, Addr, BankMsg, CosmosMsg, ReplyOn, StdError, Uint128};
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
#[messages(ibc_callbacks as IbcCallback)]
#[messages(receive as Receive)]
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
    pub fn increment(&self, mut ctx: &mut AppExecCtx, i: u64) -> Result<(), AppError> {
        let counter = self.counter.load(ctx.deps.storage)?;
        ensure!(counter < 10, StdError::generic_err("Limit reached"));
        ensure!(i < 10, StdError::generic_err("Unauthorized"));

        self.counter.save(ctx.deps.storage, &(counter + 1))?;

        let amount = coins(145, "ujuno");

        ctx.bank()
            .with_reply(ReplyOn::Always, 76)
            .transfer(amount.clone(), &Addr::unchecked("adair".to_string()))?;

        ctx.add_attribute("action", "execute_test");

        Ok(())
    }

    #[msg(query)]
    pub fn count(&self, ctx: &AppQueryCtx) -> Result<u64, AppError> {
        Ok(self.counter.load(ctx.deps.storage)?)
    }
}

impl AbstractAppBase for SylviaContract<'_> {
    type Error = AppError;
    const INFO: ModuleStateInfo = ModuleStateInfo::new("sylvia:counter", "1.0.1", Some("Metadata"));
    const DEPENDENCIES: &'static [abstract_core::objects::dependency::StaticDependency] = &[];
}

pub mod ibc_callbacks {
    use abstract_app::{
        better_sdk::{
            contexts::{AppExecCtx, AppQueryCtx},
            sdk::BaseIbcCallback,
        },
        AppError,
    };
    use abstract_sdk::AbstractSdkError;
    use cosmwasm_std::{Response, StdError};
    use sylvia::interface;

    #[interface]
    /// Allows to register any function associated with the trait to call before execution
    #[base_exec(base_ibc)]
    pub trait IbcCallback: BaseIbcCallback {
        type Error: From<StdError> + From<AppError>;

        #[msg(exec)]
        fn dex_callback(
            &self,
            _ctx: &mut AppExecCtx,
            _callback_msg: Option<cosmwasm_std::Binary>,
            _result: polytone::callbacks::Callback,
        ) -> Result<(), AppError> {
            Ok(())
        }

        #[msg(exec)]
        fn random_callback(
            &self,
            _ctx: &mut AppExecCtx,
            _callback_msg: Option<cosmwasm_std::Binary>,
            _result: polytone::callbacks::Callback,
        ) -> Result<(), AppError> {
            Ok(())
        }
    }
}
impl ibc_callbacks::IbcCallback for SylviaContract<'_> {
    type Error = AppError;
}

pub mod receive {
    use abstract_app::{better_sdk::contexts::AppExecCtx, AppError};
    use cosmwasm_std::{StdError, Uint128};
    use sylvia::interface;

    #[interface]
    pub trait Receive {
        type Error: From<StdError> + From<AppError>;

        #[msg(exec)]
        fn cw20(
            &self,
            _ctx: &mut AppExecCtx,
            amount: Uint128,
            sender: String,
        ) -> Result<(), AppError> {
            Ok(())
        }

        #[msg(exec)]
        fn cw721(
            &self,
            _ctx: &mut AppExecCtx,
            token_id: String,
            sender: String,
        ) -> Result<(), AppError> {
            Ok(())
        }
    }
}
impl receive::Receive for SylviaContract<'_> {
    type Error = AppError;
}

pub mod interface {

    use abstract_app::better_sdk::sdk::AbstractAppBase;
    use cosmwasm_std::Empty;
    use cw_orch::{contract::interface_traits::Uploadable, mock::Mock, prelude::ContractWrapper};

    use super::entry_points::{execute, instantiate, migrate, query};
    use super::sv::{ExecMsg, InstantiateMsg, QueryMsg};

    use crate::SylviaContract;

    #[cw_orch::interface(InstantiateMsg, ExecMsg, QueryMsg, Empty)]
    pub struct SylviaApp;

    impl ::abstract_interface::AppDeployer<Mock> for SylviaApp<Mock> {}

    impl Uploadable for SylviaApp<Mock> {
        fn wrapper(&self) -> <Mock as ::cw_orch::environment::TxHandler>::ContractSource {
            Box::new(
                ContractWrapper::<_, _, _, _, _, _>::new_with_empty(execute, instantiate, query)
                    .with_migrate(migrate),
            )
        }
    }

    impl<Chain: ::cw_orch::environment::CwEnv> SylviaApp<Chain> {
        pub fn new_test(chain: Chain) -> Self {
            Self(cw_orch::contract::Contract::new(
                <SylviaContract as AbstractAppBase>::INFO.name,
                chain,
            ))
        }
    }
}

fn main() {
    test::main().unwrap();
    test::receive().unwrap();
    test::ibc().unwrap();
    test::execute_test().unwrap();
}

pub mod test {

    use crate::entry_points::{execute, instantiate, query};
    use crate::ibc_callbacks::sv::IbcCallbackExecMsg;
    use crate::interface::SylviaApp;
    use crate::receive::sv::ReceiveExecMsg;
    use crate::sv::{
        ContractExecMsg, ContractQueryMsg, ExecMsg, ImplInstantiateMsg, InstantiateMsg, QueryMsg,
    };
    use crate::SylviaContract;

    use abstract_app::better_sdk::sdk::sv::AbstractAppBaseExecMsg;
    use abstract_app::better_sdk::sdk::AbstractAppBase;
    use abstract_core::app::BaseInstantiateMsg;
    use abstract_core::objects::account::TEST_ACCOUNT_ID;
    use abstract_core::objects::gov_type::GovernanceDetails;
    use abstract_interface::{
        Abstract, AbstractAccount, AccountFactory, AppDeployer, DeployStrategy, VCExecFns,
    };
    use abstract_testing::addresses::{
        TEST_ANS_HOST, TEST_MANAGER, TEST_MODULE_FACTORY, TEST_VERSION_CONTROL,
    };
    use abstract_testing::mock_querier;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_json, Addr, Attribute, Uint128};
    use cw_orch::anyhow;
    use cw_orch::deploy::Deploy;
    use cw_orch::mock::Mock;

    const OWNER: &str = "owner";

    pub(crate) fn create_default_account(
        factory: &AccountFactory<Mock>,
    ) -> anyhow::Result<AbstractAccount<Mock>> {
        let account = factory.create_default_account(GovernanceDetails::Monarchy {
            monarch: Addr::unchecked(OWNER).to_string(),
        })?;
        Ok(account)
    }

    pub fn main() -> anyhow::Result<()> {
        let sender = Addr::unchecked(OWNER);
        let chain = Mock::new(&sender);
        let amount = coins(100_000_000, "ujuno");
        chain.set_balance(&sender, amount.clone())?;
        let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
        let account = create_default_account(&deployment.account_factory)?;

        deployment
            .version_control
            .claim_namespace(TEST_ACCOUNT_ID, "sylvia".to_owned())?;

        let app = SylviaApp::new_test(chain);
        app.deploy(
            SylviaContract::INFO.version.parse().unwrap(),
            DeployStrategy::Try,
        )?;
        let app_addr =
            account.install_app(app, &ImplInstantiateMsg { couter_init: 8 }, Some(&amount))?;

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
        Ok(())
    }

    pub fn receive() -> cw_orch::anyhow::Result<()> {
        let mut deps = mock_dependencies();
        deps.querier = mock_querier();
        let response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info(TEST_MODULE_FACTORY, &[]),
            ContractExecMsg::Receive(ReceiveExecMsg::Cw20 {
                amount: Uint128::one(),
                sender: "abstract-account".to_string(),
            }),
        )
        .unwrap();

        Ok(())
    }

    pub fn ibc() -> cw_orch::anyhow::Result<()> {
        let mut deps = mock_dependencies();
        deps.querier = mock_querier();
        let response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info(TEST_MODULE_FACTORY, &[]),
            ContractExecMsg::IbcCallback(IbcCallbackExecMsg::DexCallback {
                _callback_msg: None,
                _result: polytone::callbacks::Callback::FatalError(
                    "Fatal error, please take a look ".to_string(),
                ),
            }),
        )
        .unwrap();

        Ok(())
    }

    pub fn execute_test() -> cw_orch::anyhow::Result<()> {
        let mut deps = mock_dependencies();
        deps.querier = mock_querier();
        let response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info(TEST_MODULE_FACTORY, &[]),
            ContractExecMsg::SylviaContract(ExecMsg::Increment { i: 7 }),
        )
        .unwrap();

        Ok(())
    }
}
