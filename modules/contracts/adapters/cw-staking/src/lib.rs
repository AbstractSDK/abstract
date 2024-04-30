mod adapter;
pub mod contract;
mod handlers;

mod resolver;

pub mod msg {
    pub use abstract_staking_standard::msg::*;
}

pub use abstract_staking_standard::{CwStakingCommand, CW_STAKING_ADAPTER_ID};

#[cfg(any(feature = "juno", feature = "osmosis"))]
pub mod host_staking {
    pub use abstract_osmosis_adapter::staking::Osmosis;
}

pub use abstract_staking_standard::error;

#[cfg(feature = "testing")]
pub mod staking_tester;

#[cfg(feature = "interface")]
pub use msg::StakingQueryMsgFns;

#[cfg(feature = "interface")]
pub mod interface {
    use abstract_adapter::abstract_interface::{
        AbstractAccount, AbstractInterfaceError, AdapterDeployer, RegisteredModule,
    };
    use abstract_adapter::sdk::{base::Handler, features::ModuleIdentification as _};
    use abstract_adapter::std::{
        adapter,
        objects::{AnsAsset, AssetEntry},
    };
    use cw_orch::{build::BuildPostfix, contract::Contract, interface, prelude::*};

    use crate::{
        contract::CW_STAKING_ADAPTER,
        msg::{ExecuteMsg, InstantiateMsg, QueryMsg, StakingAction, StakingExecuteMsg},
    };

    /// Contract wrapper for interacting with BOOT
    #[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
    pub struct CwStakingAdapter<Chain>;

    impl<Chain: CwEnv> AdapterDeployer<Chain, Empty> for CwStakingAdapter<Chain> {}

    impl<Chain: CwEnv> Uploadable for CwStakingAdapter<Chain> {
        fn wrapper(&self) -> <Mock as TxHandler>::ContractSource {
            Box::new(ContractWrapper::new_with_empty(
                crate::contract::execute,
                crate::contract::instantiate,
                crate::contract::query,
            ))
        }
        fn wasm(&self) -> WasmPath {
            artifacts_dir_from_workspace!()
                .find_wasm_path_with_build_postfix(
                    "abstract_cw_staking",
                    BuildPostfix::<Chain>::ChainName(self.get_chain()),
                )
                .unwrap()
        }
    }

    impl<Chain: CwEnv> RegisteredModule for CwStakingAdapter<Chain> {
        type InitMsg = <crate::contract::CwStakingAdapter as Handler>::CustomInitMsg;

        fn module_id<'a>() -> &'a str {
            CW_STAKING_ADAPTER.module_id()
        }

        fn module_version<'a>() -> &'a str {
            CW_STAKING_ADAPTER.version()
        }
    }

    impl<Chain: CwEnv> From<Contract<Chain>> for CwStakingAdapter<Chain> {
        fn from(contract: Contract<Chain>) -> Self {
            Self(contract)
        }
    }
    /// implement chain-generic functions
    impl<Chain: CwEnv> CwStakingAdapter<Chain>
    where
        TxResponse<Chain>: IndexResponse,
    {
        /// Staking action using Abstract Account
        pub fn staking_action(
            &self,
            provider: String,
            action: StakingAction,
            account: impl AsRef<AbstractAccount<Chain>>,
        ) -> Result<<Chain as TxHandler>::Response, AbstractInterfaceError> {
            let account = account.as_ref();
            let swap_msg = crate::msg::ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: Some(account.proxy.addr_str()?),
                request: StakingExecuteMsg { provider, action },
            });
            self.execute(&swap_msg, None).map_err(Into::into)
        }

        /// Stake using Abstract Account (registered in daemon_state).
        pub fn stake(
            &self,
            stake_asset: AnsAsset,
            provider: String,
            duration: Option<cw_utils::Duration>,
            account: impl AsRef<AbstractAccount<Chain>>,
        ) -> Result<(), AbstractInterfaceError> {
            let action = StakingAction::Stake {
                assets: vec![stake_asset],
                unbonding_period: duration,
            };
            self.staking_action(provider, action, account)?;
            Ok(())
        }

        pub fn unstake(
            &self,
            stake_asset: AnsAsset,
            provider: String,
            duration: Option<cw_utils::Duration>,
            account: impl AsRef<AbstractAccount<Chain>>,
        ) -> Result<(), AbstractInterfaceError> {
            let action = StakingAction::Unstake {
                assets: vec![stake_asset],
                unbonding_period: duration,
            };
            self.staking_action(provider, action, account)?;
            Ok(())
        }

        pub fn claim(
            &self,
            stake_asset: AssetEntry,
            provider: String,
            account: impl AsRef<AbstractAccount<Chain>>,
        ) -> Result<(), AbstractInterfaceError> {
            let action = StakingAction::Claim {
                assets: vec![stake_asset],
            };
            self.staking_action(provider, action, account)?;
            Ok(())
        }

        pub fn claim_rewards(
            &self,
            stake_asset: AssetEntry,
            provider: String,
            account: impl AsRef<AbstractAccount<Chain>>,
        ) -> Result<(), AbstractInterfaceError> {
            let action = StakingAction::ClaimRewards {
                assets: vec![stake_asset],
            };
            self.staking_action(provider, action, account)?;
            Ok(())
        }
    }
}
