#[macro_export]
macro_rules! gen_app_better_mock {
    ($name:ident,$id:expr, $version:expr, $deps:expr) => {
        use ::abstract_app::better_sdk::{
            account_identification::AccountIdentification,
            bank::TransferInterface,
            contexts::{AppExecCtx, AppInstantiateCtx, AppMigrateCtx, AppQueryCtx},
            execution_stack::CustomData,
            sdk::AbstractAppBase,
            sdk::ContractInfo,
        };
        use ::abstract_app::mock::{
            MockExecMsg, MockInitMsg, MockMigrateMsg, MockQueryMsg, MockReceiveMsg,
        };
        use ::abstract_app::AppError;
        use ::abstract_core::app;
        use ::abstract_sdk::base::Handler;
        use ::cw_orch::prelude::*;
        use ::sylvia::{contract, entry_points};

        pub struct $name<'a> {
            _marker: std::marker::PhantomData<&'a ()>,
        }

        impl Default for $name<'_> {
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
        impl $name<'_> {
            pub fn new() -> Self {
                Self::default()
            }

            #[msg(instantiate)]
            pub fn instantiate<'a>(
                &self,
                mut ctx: AppInstantiateCtx<'a>,
            ) -> Result<AppInstantiateCtx<'a>, AppError> {
                ctx.set_data("mock_init".as_bytes());
                // See test `create_sub_account_with_installed_module` where this will be triggered.
                if Self::INFO.0 == "tester:mock-app1" {
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
                    Ok(ctx)
                } else {
                    Ok(ctx)
                }
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
            fn mock_migrate<'a>(
                &self,
                mut ctx: AppMigrateCtx<'a>,
            ) -> Result<AppMigrateCtx<'a>, AppError> {
                Ok(ctx)
            }
        }

        impl AbstractAppBase for $name<'_> {
            type Error = AppError;
            const INFO: ContractInfo = ($id, $version, None);
            const DEPENDENCIES: &'static [abstract_core::objects::dependency::StaticDependency] =
                $deps;
        }
    };
}
