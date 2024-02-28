use crate::{interface::DexAdapter, msg::DexInstantiateMsg, DEX_ADAPTER_ID};
use abstract_client::{AbstractClient, Environment};
use abstract_core::objects::{
    module::{ModuleInfo, ModuleVersion},
    pool_id::PoolAddressBase,
    PoolMetadata,
};
use abstract_interface::{Abstract, AdapterDeployer, DeployStrategy, ExecuteMsgFns, VCExecFns};
use cosmwasm_std::{coins, Decimal, Uint128};
use cw_asset::AssetInfoUnchecked;
use cw_orch::{anyhow, environment::MutCwEnv, prelude::*};

pub struct DexTester<Chain: MutCwEnv, Dex: MockDex> {
    pub abstr_deployment: AbstractClient<Chain>,
    pub dex_adapter: DexAdapter<Chain>,
    pub dex: Dex,
}

impl<Chain: MutCwEnv, Dex: MockDex> DexTester<Chain, Dex> {
    pub fn new(abstr_deployment: AbstractClient<Chain>, dex: Dex) -> anyhow::Result<Self> {
        // Re-register dex, to make sure it's latest
        let _ = abstr_deployment
            .version_control()
            .remove_module(ModuleInfo::from_id(
                DEX_ADAPTER_ID,
                ModuleVersion::Version(crate::contract::CONTRACT_VERSION.to_owned()),
            )?);
        let dex_adapter = DexAdapter::new(DEX_ADAPTER_ID, abstr_deployment.environment());
        dex_adapter.deploy(
            crate::contract::CONTRACT_VERSION.parse()?,
            DexInstantiateMsg {
                recipient_account: 0,
                swap_fee: Decimal::permille(3),
            },
            DeployStrategy::Force,
        )?;

        {
            let pool = dex.create_pool()?;
            let abstr = Abstract::load_from(abstr_deployment.environment())?;
            // Add assets
            abstr
                .ans_host
                .update_asset_addresses(vec![dex.asset_a(), dex.asset_b()], vec![])?;
            // Add dex
            abstr.ans_host.update_dexes(vec![dex.name()], vec![])?;
            // Add pool
            abstr.ans_host.update_pools(vec![pool], vec![])?;
        }

        Ok(Self {
            abstr_deployment,
            dex_adapter,
            dex,
        })
    }

    pub fn test_swap(&self) -> anyhow::Result<()> {
        let (ans_asset_a, asset_info_a) = self.dex.asset_a();
        let (ans_asset_b, asset_info_b) = self.dex.asset_b();

        let new_account = self
            .abstr_deployment
            .account_builder()
            .install_adapter::<DexAdapter<Chain>>()?
            .build()?;
        let proxy_addr = new_account.proxy()?;

        // swap 1_000_000_000 asset_a to asset_b
        let swap_value = 1_000_000_000u128;

        self.add_proxy_balance(&proxy_addr, &asset_info_a, swap_value)?;

        self.dex_adapter.ans_swap(
            (&ans_asset_a, swap_value),
            &ans_asset_b,
            self.dex.name(),
            new_account.as_ref(),
        )?;

        // Assert balances
        let balance_a = self.query_proxy_balance(&proxy_addr, &asset_info_a)?;
        assert!(balance_a.is_zero());
        let balance_b = self.query_proxy_balance(&proxy_addr, &asset_info_b)?;
        assert!(!balance_b.is_zero());

        // swap balance_b asset_b to asset_a
        self.dex_adapter.ans_swap(
            (&ans_asset_b, balance_b.u128()),
            &ans_asset_a,
            self.dex.name(),
            new_account.as_ref(),
        )?;

        // Assert balances
        let balance_a = self.query_proxy_balance(&proxy_addr, &asset_info_a)?;
        assert!(!balance_a.is_zero());
        let balance_b = self.query_proxy_balance(&proxy_addr, &asset_info_b)?;
        assert!(balance_b.is_zero());

        Ok(())
    }

    fn add_proxy_balance(
        &self,
        proxy_addr: &Addr,
        asset: &AssetInfoUnchecked,
        amount: u128,
    ) -> anyhow::Result<()> {
        let mut chain = self.abstr_deployment.environment();

        match asset {
            cw_asset::AssetInfoBase::Native(denom) => {
                chain.add_balance(proxy_addr, coins(amount, denom))?;
            }
            cw_asset::AssetInfoBase::Cw20(addr) => {
                chain.execute(
                    &cw20::Cw20ExecuteMsg::Mint {
                        recipient: proxy_addr.to_string(),
                        amount: amount.into(),
                    },
                    &[],
                    &Addr::unchecked(addr),
                )?;
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn query_proxy_balance(
        &self,
        proxy_addr: &Addr,
        asset: &AssetInfoUnchecked,
    ) -> anyhow::Result<Uint128> {
        let chain = self.abstr_deployment.environment();

        let balance = match asset {
            cw_asset::AssetInfoBase::Native(denom) => {
                chain
                    .bank_querier()
                    .balance(proxy_addr, Some(denom.to_owned()))
                    .unwrap()
                    .pop()
                    .unwrap()
                    .amount
            }
            cw_asset::AssetInfoBase::Cw20(addr) => {
                let balance: cw20::BalanceResponse = chain
                    .query(
                        &cw20::Cw20QueryMsg::Balance {
                            address: proxy_addr.to_string(),
                        },
                        &Addr::unchecked(addr),
                    )
                    .unwrap();
                balance.balance
            }
            _ => unreachable!(),
        };

        Ok(balance)
    }
}

pub trait MockDex {
    /// Name of the dex
    fn name(&self) -> String;

    /// First asset
    fn asset_a(&self) -> (String, cw_asset::AssetInfoUnchecked);

    /// Second asset
    fn asset_b(&self) -> (String, cw_asset::AssetInfoUnchecked);

    /// Create pool with asset_a and asset_b
    fn create_pool(&self) -> anyhow::Result<(PoolAddressBase<String>, PoolMetadata)>;
}
