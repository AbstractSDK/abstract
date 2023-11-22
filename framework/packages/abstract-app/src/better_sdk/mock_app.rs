#[macro_export]
macro_rules! gen_app_better_mock {
    ($name:ident,$id:expr, $version:expr, $deps:expr) => {
        use ::abstract_app::better_sdk::{
            account_identification::AccountIdentification,
            module_identification::ModuleIdentification,
            bank::TransferInterface,
            contexts::{AppExecCtx, AppInstantiateCtx, AppMigrateCtx, AppQueryCtx},
            execution_stack::CustomData,
            sdk::AbstractAppBase,
            sdk::ModuleStateInfo,
        };
        use ::abstract_app::mock::{
            MockExecMsg, MockInitMsg, MockMigrateMsg, MockQueryMsg, MockReceiveMsg,
        };
        use ::abstract_app::AppError;
        use ::abstract_core::app;
        use ::abstract_sdk::base::Handler;
        use ::cw_orch::prelude::*;
        use ::sylvia::{contract, entry_points};

        pub struct Contract<'a> {
            _marker: std::marker::PhantomData<&'a ()>,
        }

        impl Default for Contract<'_> {
            fn default() -> Self {
                Self {
                    _marker: std::marker::PhantomData,
                }
            }
        }

        #[entry_points]
        #[contract]
        #[contract_type(::abstract_app::better_sdk::sdk::AbstractApp)]
        #[error(::abstract_app::AppError)]
        #[messages(::abstract_app::better_sdk::sdk as Base)]
        impl Contract<'_> {
            pub fn new() -> Self {
                Self::default()
            }

            #[msg(instantiate)]
            pub fn instantiate(&self, ctx: &mut AppInstantiateCtx) -> Result<(), AppError> {
                ctx.set_data("mock_init".as_bytes());
                // See test `create_sub_account_with_installed_module` where this will be triggered.
                if ctx.module_id()?.eq("tester:mock-app1") {
                    println!("checking address of adapter1");
                    let manager = self.admin().get(ctx.deps.as_ref())?.unwrap();
                    // Check if the adapter has access to its dependency during instantiation.
                    let adapter1_addr = ::abstract_core::manager::state::ACCOUNT_MODULES.query(
                        &ctx.deps.querier,
                        manager,
                        "tester:mock-adapter1",
                    )?;
                    // We have address!
                    ::cosmwasm_std::ensure!(
                        adapter1_addr.is_some(),
                        ::cosmwasm_std::StdError::generic_err("no address")
                    );
                    println!("adapter_addr: {adapter1_addr:?}");
                    // See test `install_app_with_proxy_action` where this transfer will happen.
                    let proxy_addr = ctx.proxy_address()?;
                    let balance = ctx.deps.querier.query_balance(proxy_addr, "TEST")?;
                    if !balance.amount.is_zero() {
                        println!("sending amount from proxy: {balance:?}");
                        ctx.bank().transfer::<::cosmwasm_std::Coin>(
                            vec![balance.into()],
                            &::cosmwasm_std::Addr::unchecked("test_addr"),
                        )?;
                    }
                }
                Ok(())
            }

            #[msg(exec)]
            fn mock_exec<'a>(&self, mut ctx: AppExecCtx<'a>) -> Result<AppExecCtx<'a>, AppError> {
                Ok(ctx)
            }

            #[msg(query)]
            fn mock_query(&self, mut ctx: AppQueryCtx) -> Result<String, AppError> {
                Ok("mock_query".to_string())
            }

            #[msg(migrate)]
            fn mock_migrate<'a>(&self, ctx: &mut AppMigrateCtx<'a>) -> Result<(), AppError> {
                Ok(())
            }
        }

        impl AbstractAppBase for Contract<'_> {
            type Error = AppError;
            const INFO: ModuleStateInfo = ModuleStateInfo{
                name: $id,
                version: $version,
                metadata: None
            };
            const DEPENDENCIES: &'static [abstract_core::objects::dependency::StaticDependency] =
                $deps;
        }

        use ::cosmwasm_std::Empty;
        use ::cw_orch::{
            contract::interface_traits::Uploadable, mock::Mock, prelude::ContractWrapper,
        };

        use entry_points::{execute, instantiate, migrate, query};
        use sv::{ExecMsg, InstantiateMsg, QueryMsg};

        #[cw_orch::interface(InstantiateMsg, ExecMsg, QueryMsg, Empty)]
        pub struct $name;

        impl ::abstract_interface::AppDeployer<Mock> for $name<Mock> {}

        impl Uploadable for $name<Mock> {
            fn wrapper(&self) -> <Mock as ::cw_orch::environment::TxHandler>::ContractSource {
                Box::new(
                    ContractWrapper::<_, _, _, _, _, _>::new_with_empty(
                        execute,
                        instantiate,
                        query,
                    )
                    .with_migrate(migrate),
                )
            }
        }

        impl<Chain: ::cw_orch::environment::CwEnv> $name<Chain> {
            pub fn new_test(chain: Chain) -> Self {
                Self(cw_orch::contract::Contract::new($id, chain))
            }
        }
    };
}
