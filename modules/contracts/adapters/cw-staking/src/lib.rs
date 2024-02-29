mod adapter;
pub mod contract;
mod handlers;

mod resolver;

pub mod msg {
    pub use abstract_staking_standard::msg::*;
}

pub use abstract_staking_standard::{CwStakingCommand, CW_STAKING_ADAPTER_ID};
pub use adapter::CwStakingAdapter;

#[cfg(any(feature = "juno", feature = "osmosis"))]
pub mod host_staking {
    pub use abstract_osmosis_adapter::staking::Osmosis;
}

pub use abstract_staking_standard::error;

#[cfg(feature = "testing")]
pub mod staking_tester;

#[cfg(feature = "interface")]
pub mod interface {
    use abstract_core::{
        adapter,
        objects::{AnsAsset, AssetEntry},
    };
    use abstract_interface::{
        AbstractAccount, AbstractInterfaceError, AdapterDeployer, RegisteredModule,
    };
    use abstract_sdk::{base::Handler, features::ModuleIdentification as _};
    use cosmwasm_std::{Addr, Empty};
    use cw_orch::{build::BuildPostfix, contract::Contract, interface, prelude::*};

    use crate::{
        contract::CW_STAKING_ADAPTER,
        msg::{ExecuteMsg, InstantiateMsg, QueryMsg, StakingAction, StakingExecuteMsg},
        CW_STAKING_ADAPTER_ID,
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
        pub fn load(chain: Chain, addr: &Addr) -> Self {
            Self(Contract::new(CW_STAKING_ADAPTER_ID, chain).with_address(Some(addr)))
        }

        /// Swap using Abstract's OS (registered in daemon_state).
        pub fn stake(
            &self,
            stake_asset: AnsAsset,
            provider: String,
            duration: Option<cw_utils::Duration>,
            account: &AbstractAccount<Chain>,
        ) -> Result<(), AbstractInterfaceError> {
            let stake_msg = ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: None,
                request: StakingExecuteMsg {
                    provider,
                    action: StakingAction::Stake {
                        assets: vec![stake_asset],
                        unbonding_period: duration,
                    },
                },
            });
            account
                .manager
                .execute_on_module(CW_STAKING_ADAPTER_ID, stake_msg)?;
            Ok(())
        }

        pub fn unstake(
            &self,
            stake_asset: AnsAsset,
            provider: String,
            duration: Option<cw_utils::Duration>,
            account: &AbstractAccount<Chain>,
        ) -> Result<(), AbstractInterfaceError> {
            let stake_msg = ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: None,
                request: StakingExecuteMsg {
                    provider,
                    action: StakingAction::Unstake {
                        assets: vec![stake_asset],
                        unbonding_period: duration,
                    },
                },
            });
            account
                .manager
                .execute_on_module(CW_STAKING_ADAPTER_ID, stake_msg)?;
            Ok(())
        }

        pub fn claim(
            &self,
            stake_asset: AssetEntry,
            provider: String,
            account: &AbstractAccount<Chain>,
        ) -> Result<(), AbstractInterfaceError> {
            let claim_msg = ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: None,
                request: StakingExecuteMsg {
                    provider,
                    action: StakingAction::Claim {
                        assets: vec![stake_asset],
                    },
                },
            });
            account
                .manager
                .execute_on_module(CW_STAKING_ADAPTER_ID, claim_msg)?;
            Ok(())
        }

        pub fn claim_rewards(
            &self,
            stake_asset: AssetEntry,
            provider: String,
            account: &AbstractAccount<Chain>,
        ) -> Result<(), AbstractInterfaceError> {
            let claim_rewards_msg = ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: None,
                request: StakingExecuteMsg {
                    provider,
                    action: StakingAction::ClaimRewards {
                        assets: vec![stake_asset],
                    },
                },
            });
            account
                .manager
                .execute_on_module(CW_STAKING_ADAPTER_ID, claim_rewards_msg)?;
            Ok(())
        }
    }
}
