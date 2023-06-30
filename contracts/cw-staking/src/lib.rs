mod adapter;
pub mod contract;
mod handlers;

pub mod providers;
mod resolver;

pub use abstract_staking_adapter_traits::msg;
pub use abstract_staking_adapter_traits::CwStakingCommand;
pub use adapter::CwStakingAdapter;

pub const CW_STAKING: &str = "abstract:cw-staking";

#[cfg(any(feature = "juno", feature = "osmosis"))]
pub mod host_staking {
    pub use abstract_osmosis_adapter::staking::Osmosis;
}

pub use abstract_staking_adapter_traits::error;

#[cfg(feature = "interface")]
pub mod interface {
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, StakingAction, StakingExecuteMsg};
    use crate::CW_STAKING;
    use abstract_core::objects::{AnsAsset, AssetEntry};
    use abstract_core::{adapter, MANAGER};
    use abstract_interface::AbstractInterfaceError;
    use abstract_interface::AdapterDeployer;
    use abstract_interface::Manager;
    use cosmwasm_std::{Addr, Empty};
    use cw_orch::contract::Contract;
    use cw_orch::interface;
    use cw_orch::prelude::*;

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
                .find_wasm_path("abstract_cw_staking")
                .unwrap()
        }
    }

    /// implement chain-generic functions
    impl<Chain: CwEnv> CwStakingAdapter<Chain>
    where
        TxResponse<Chain>: IndexResponse,
    {
        pub fn load(chain: Chain, addr: &Addr) -> Self {
            Self(Contract::new(CW_STAKING, chain).with_address(Some(addr)))
        }

        /// Swap using Abstract's OS (registered in daemon_state).
        pub fn stake(
            &self,
            stake_asset: AnsAsset,
            provider: String,
            duration: Option<cw_utils::Duration>,
        ) -> Result<(), AbstractInterfaceError> {
            let manager = Manager::new(MANAGER, self.get_chain().clone());
            let stake_msg = ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: None,
                request: StakingExecuteMsg {
                    provider,
                    action: StakingAction::Stake {
                        asset: stake_asset,
                        unbonding_period: duration,
                    },
                },
            });
            manager.execute_on_module(CW_STAKING, stake_msg)?;
            Ok(())
        }

        pub fn unstake(
            &self,
            stake_asset: AnsAsset,
            provider: String,
            duration: Option<cw_utils::Duration>,
        ) -> Result<(), AbstractInterfaceError> {
            let manager = Manager::new(MANAGER, self.get_chain().clone());
            let stake_msg = ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: None,
                request: StakingExecuteMsg {
                    provider,
                    action: StakingAction::Unstake {
                        asset: stake_asset,
                        unbonding_period: duration,
                    },
                },
            });
            manager.execute_on_module(CW_STAKING, stake_msg)?;
            Ok(())
        }

        pub fn claim(
            &self,
            stake_asset: AssetEntry,
            provider: String,
        ) -> Result<(), AbstractInterfaceError> {
            let manager = Manager::new(MANAGER, self.get_chain().clone());
            let claim_msg = ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: None,
                request: StakingExecuteMsg {
                    provider,
                    action: StakingAction::Claim { asset: stake_asset },
                },
            });
            manager.execute_on_module(CW_STAKING, claim_msg)?;
            Ok(())
        }

        pub fn claim_rewards(
            &self,
            stake_asset: AssetEntry,
            provider: String,
        ) -> Result<(), AbstractInterfaceError> {
            let manager = Manager::new(MANAGER, self.get_chain().clone());
            let claim_rewards_msg = ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: None,
                request: StakingExecuteMsg {
                    provider,
                    action: StakingAction::ClaimRewards { asset: stake_asset },
                },
            });
            manager.execute_on_module(CW_STAKING, claim_rewards_msg)?;
            Ok(())
        }
    }
}
