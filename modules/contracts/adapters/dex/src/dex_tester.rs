use crate::{interface::DexAdapter, msg::DexInstantiateMsg, DEX_ADAPTER_ID};
use abstract_adapter::abstract_interface::{
    AdapterDeployer, DeployStrategy, ExecuteMsgFns, RegistryExecFns,
};
use abstract_adapter::std::objects::{
    module::{ModuleInfo, ModuleVersion},
    pool_id::PoolAddressBase,
    AnsAsset, AssetEntry, LpToken, PoolMetadata,
};
use abstract_client::{AbstractClient, ClientResolve, Environment};
use abstract_dex_standard::ans_action::WholeDexAction;
use abstract_dex_standard::{
    ans_action::DexAnsAction,
    msg::{DexFeesResponse, DexQueryMsg, GenerateMessagesResponse, SimulateSwapResponse},
};
use cosmwasm_std::{coins, from_json, BankMsg, CosmosMsg, Decimal, Uint128, WasmMsg};
use cw_asset::AssetInfoUnchecked;
use cw_orch::{environment::MutCwEnv, prelude::*};

use cw_orch::anyhow;

pub trait MockDex {
    /// Name of the dex
    fn name(&self) -> String;

    /// First asset
    fn asset_a(&self) -> (String, cw_asset::AssetInfoUnchecked);

    /// Second asset
    fn asset_b(&self) -> (String, cw_asset::AssetInfoUnchecked);

    /// Create pool with asset_a and asset_b
    /// Returns Pool Entry for ANS and LP asset
    fn create_pool(
        &self,
    ) -> anyhow::Result<(PoolAddressBase<String>, PoolMetadata, AssetInfoUnchecked)>;
}

pub struct DexTester<Chain: MutCwEnv, Dex: MockDex> {
    pub abstr_deployment: AbstractClient<Chain>,
    pub dex_adapter: DexAdapter<Chain>,
    pub dex: Dex,
    pub lp_asset: AssetInfoUnchecked,
}

impl<Chain: MutCwEnv, Dex: MockDex> DexTester<Chain, Dex> {
    pub fn new(abstr_deployment: AbstractClient<Chain>, dex: Dex) -> anyhow::Result<Self> {
        // Re-register dex, to make sure it's latest
        let _ = abstr_deployment
            .registry()
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

        let lp_asset = {
            let (pool, pool_metadata, lp_asset) = dex.create_pool()?;
            // Add assets
            abstr_deployment
                .name_service()
                .update_asset_addresses(vec![dex.asset_a(), dex.asset_b()], vec![])?;
            // Add dex
            abstr_deployment
                .name_service()
                .update_dexes(vec![dex.name()], vec![])?;
            // Add pool
            abstr_deployment
                .name_service()
                .update_pools(vec![(pool, pool_metadata)], vec![])?;
            // Add lp asset
            let lp_token = LpToken::new(dex.name(), vec![dex.asset_a().0, dex.asset_b().0]);
            abstr_deployment
                .name_service()
                .update_asset_addresses(vec![(lp_token.to_string(), lp_asset.clone())], vec![])?;
            lp_asset
        };

        Ok(Self {
            abstr_deployment,
            dex_adapter,
            dex,
            lp_asset,
        })
    }

    pub fn test_swap(&self) -> anyhow::Result<()> {
        let (ans_asset_a, asset_info_a) = self.dex.asset_a();
        let (ans_asset_b, asset_info_b) = self.dex.asset_b();

        let new_account = self
            .abstr_deployment
            .account_builder()
            .install_adapter::<DexAdapter<Chain>>()
            .build()?;
        let account_addr = new_account.address()?;

        let swap_value = 1_000_000_000u128;

        self.add_account_balance(&account_addr, &asset_info_a, swap_value)?;

        // swap 1_000_000_000 asset_a to asset_b
        self.dex_adapter.ans_action(
            self.dex.name(),
            DexAnsAction::Swap {
                offer_asset: AnsAsset::new(AssetEntry::new(&ans_asset_a), swap_value),
                ask_asset: AssetEntry::new(&ans_asset_b),
                max_spread: None,
                belief_price: None,
            },
            &new_account,
            self.abstr_deployment.name_service(),
        )?;

        // Assert balances
        let balance_a = self.query_addr_balance(&account_addr, &asset_info_a)?;
        assert!(balance_a.is_zero());
        let balance_b = self.query_addr_balance(&account_addr, &asset_info_b)?;
        assert!(!balance_b.is_zero());

        // swap balance_b asset_b to asset_a
        self.dex_adapter.ans_action(
            self.dex.name(),
            DexAnsAction::Swap {
                offer_asset: AnsAsset::new(AssetEntry::new(&ans_asset_b), balance_b),
                ask_asset: AssetEntry::new(&ans_asset_a),
                max_spread: None,
                belief_price: None,
            },
            &new_account,
            self.abstr_deployment.name_service(),
        )?;

        // Assert balances
        let balance_a = self.query_addr_balance(&account_addr, &asset_info_a)?;
        assert!(!balance_a.is_zero());
        let balance_b = self.query_addr_balance(&account_addr, &asset_info_b)?;
        assert!(balance_b.is_zero());

        Ok(())
    }

    pub fn test_swap_slippage(
        &self,
        belief_price_a_to_b: Decimal,
        belief_price_b_to_a: Decimal,
    ) -> anyhow::Result<()> {
        let (ans_asset_a, asset_info_a) = self.dex.asset_a();
        let (ans_asset_b, asset_info_b) = self.dex.asset_b();

        let new_account = self
            .abstr_deployment
            .account_builder()
            .install_adapter::<DexAdapter<Chain>>()
            .build()?;
        let account_addr = new_account.address()?;

        let swap_value = 1_000_000_000u128;

        self.add_account_balance(&account_addr, &asset_info_a, swap_value)?;

        // swap 1_000_000_000 asset_a to asset_b
        self.dex_adapter.ans_action(
            self.dex.name(),
            DexAnsAction::Swap {
                offer_asset: AnsAsset::new(AssetEntry::new(&ans_asset_a), swap_value),
                ask_asset: AssetEntry::new(&ans_asset_b),
                max_spread: Some(Decimal::percent(10)),
                belief_price: Some(belief_price_a_to_b),
            },
            &new_account,
            self.abstr_deployment.name_service(),
        )?;

        // Assert balances
        let balance_a = self.query_addr_balance(&account_addr, &asset_info_a)?;
        assert!(balance_a.is_zero());
        let balance_b = self.query_addr_balance(&account_addr, &asset_info_b)?;
        assert!(!balance_b.is_zero());

        // swap balance_b asset_b to asset_a
        self.dex_adapter.ans_action(
            self.dex.name(),
            DexAnsAction::Swap {
                offer_asset: AnsAsset::new(AssetEntry::new(&ans_asset_b), balance_b),
                ask_asset: AssetEntry::new(&ans_asset_a),
                max_spread: Some(Decimal::percent(10)),
                belief_price: Some(belief_price_b_to_a),
            },
            &new_account,
            self.abstr_deployment.name_service(),
        )?;

        // Assert balances
        let balance_a = self.query_addr_balance(&account_addr, &asset_info_a)?;
        assert!(!balance_a.is_zero());
        let balance_b = self.query_addr_balance(&account_addr, &asset_info_b)?;
        assert!(balance_b.is_zero());

        // And invalid slippages
        // Add account balance, to make sure it's not the case of a failure
        self.add_account_balance(&account_addr, &asset_info_a, swap_value)?;
        let res = self.dex_adapter.ans_action(
            self.dex.name(),
            DexAnsAction::Swap {
                offer_asset: AnsAsset::new(AssetEntry::new(&ans_asset_a), swap_value),
                ask_asset: AssetEntry::new(&ans_asset_b),
                max_spread: Some(Decimal::percent(10)),
                belief_price: Some(Decimal::from_ratio(1u128, 4242u128)),
            },
            &new_account,
            self.abstr_deployment.name_service(),
        );

        assert!(res.is_err());

        // swap balance_b asset_b to asset_a
        let res = self.dex_adapter.ans_action(
            self.dex.name(),
            DexAnsAction::Swap {
                offer_asset: AnsAsset::new(AssetEntry::new(&ans_asset_b), swap_value),
                ask_asset: AssetEntry::new(&ans_asset_a),
                max_spread: Some(Decimal::percent(10)),
                belief_price: Some(Decimal::from_ratio(1u128, 424242u128)),
            },
            &new_account,
            self.abstr_deployment.name_service(),
        );
        assert!(res.is_err());
        Ok(())
    }

    pub fn test_provide_liquidity_two_sided(
        &self,
        provide_value_a: Option<u128>,
        provide_value_b: Option<u128>,
    ) -> anyhow::Result<()> {
        let (ans_asset_a, asset_info_a) = self.dex.asset_a();
        let (ans_asset_b, asset_info_b) = self.dex.asset_b();

        let new_account = self
            .abstr_deployment
            .account_builder()
            .install_adapter::<DexAdapter<Chain>>()
            .build()?;
        let account_addr = new_account.address()?;

        let provide_value_a = provide_value_a.unwrap_or(1_000_000_000u128);
        let provide_value_b = provide_value_b.unwrap_or(1_000_000_000u128);

        self.add_account_balance(&account_addr, &asset_info_a, provide_value_a * 2)?;
        self.add_account_balance(&account_addr, &asset_info_b, provide_value_b * 2)?;

        let asset_entry_a = AssetEntry::new(&ans_asset_a);
        let asset_entry_b = AssetEntry::new(&ans_asset_b);

        // provide to the pool
        self.dex_adapter.ans_action(
            self.dex.name(),
            DexAnsAction::ProvideLiquidity {
                assets: vec![
                    AnsAsset::new(asset_entry_a.clone(), provide_value_a),
                    AnsAsset::new(asset_entry_b.clone(), provide_value_b),
                ],
                max_spread: Some(Decimal::percent(30)),
            },
            &new_account,
            self.abstr_deployment.name_service(),
        )?;

        let lp_balance_first = self.query_addr_balance(&account_addr, &self.lp_asset)?;
        assert!(!lp_balance_first.is_zero());

        // provide to the pool reversed
        self.dex_adapter.ans_action(
            self.dex.name(),
            DexAnsAction::ProvideLiquidity {
                assets: vec![
                    AnsAsset::new(asset_entry_b, provide_value_b),
                    AnsAsset::new(asset_entry_a, provide_value_a),
                ],
                max_spread: Some(Decimal::percent(30)),
            },
            &new_account,
            self.abstr_deployment.name_service(),
        )?;

        let lp_balance_second = self.query_addr_balance(&account_addr, &self.lp_asset)?;
        assert!(lp_balance_second > lp_balance_first);

        Ok(())
    }

    pub fn test_provide_liquidity_one_sided(&self) -> anyhow::Result<()> {
        let (ans_asset_a, asset_info_a) = self.dex.asset_a();
        let (ans_asset_b, asset_info_b) = self.dex.asset_b();

        let new_account = self
            .abstr_deployment
            .account_builder()
            .install_adapter::<DexAdapter<Chain>>()
            .build()?;
        let account_addr = new_account.address()?;

        let provide_value = 1_000_000_000_000_000u128;

        self.add_account_balance(&account_addr, &asset_info_a, provide_value)?;
        self.add_account_balance(&account_addr, &asset_info_b, provide_value)?;

        let asset_entry_a = AssetEntry::new(&ans_asset_a);
        let asset_entry_b = AssetEntry::new(&ans_asset_b);

        // provide to the pool
        self.dex_adapter.ans_action(
            self.dex.name(),
            DexAnsAction::ProvideLiquidity {
                assets: vec![
                    AnsAsset::new(asset_entry_a.clone(), provide_value),
                    AnsAsset::new(asset_entry_b.clone(), Uint128::zero()),
                ],
                max_spread: None,
            },
            &new_account,
            self.abstr_deployment.name_service(),
        )?;

        let lp_balance_first = self.query_addr_balance(&account_addr, &self.lp_asset)?;
        assert!(!lp_balance_first.is_zero());

        // provide to the pool reversed
        self.dex_adapter.ans_action(
            self.dex.name(),
            DexAnsAction::ProvideLiquidity {
                assets: vec![
                    AnsAsset::new(asset_entry_b, provide_value),
                    AnsAsset::new(asset_entry_a, Uint128::zero()),
                ],
                max_spread: None,
            },
            &new_account,
            self.abstr_deployment.name_service(),
        )?;

        let lp_balance_second = self.query_addr_balance(&account_addr, &self.lp_asset)?;
        assert!(lp_balance_second > lp_balance_first);

        Ok(())
    }

    pub fn test_provide_liquidity_one_direction(
        &self,
        asset_to_provide: AssetEntry,
    ) -> anyhow::Result<()> {
        let (ans_asset_a, asset_info_a) = self.dex.asset_a();
        let (ans_asset_b, asset_info_b) = self.dex.asset_b();

        let (asset_info, zero_ans_asset) = if asset_to_provide.as_str() == ans_asset_a {
            (asset_info_a, AssetEntry::new(&ans_asset_b))
        } else if asset_to_provide.as_str() == ans_asset_b {
            (asset_info_b, AssetEntry::new(&ans_asset_a))
        } else {
            panic!("Could not determine which asset to provide")
        };

        let new_account = self
            .abstr_deployment
            .account_builder()
            .install_adapter::<DexAdapter<Chain>>()
            .build()?;
        let account_addr = new_account.address()?;

        let provide_value = 1_000_000_000_000_000u128;

        self.add_account_balance(&account_addr, &asset_info, provide_value)?;

        // provide to the pool
        self.dex_adapter.ans_action(
            self.dex.name(),
            DexAnsAction::ProvideLiquidity {
                assets: vec![
                    AnsAsset::new(asset_to_provide, provide_value),
                    AnsAsset::new(zero_ans_asset, Uint128::zero()),
                ],
                max_spread: None,
            },
            &new_account,
            self.abstr_deployment.name_service(),
        )?;

        let lp_balance_first = self.query_addr_balance(&account_addr, &self.lp_asset)?;
        assert!(!lp_balance_first.is_zero());
        Ok(())
    }

    pub fn test_provide_liquidity_spread(&self) -> anyhow::Result<()> {
        let (ans_asset_a, asset_info_a) = self.dex.asset_a();
        let (ans_asset_b, asset_info_b) = self.dex.asset_b();

        let new_account = self
            .abstr_deployment
            .account_builder()
            .install_adapter::<DexAdapter<Chain>>()
            .build()?;
        let account_addr = new_account.address()?;

        let provide_value = 1_000_000_000u128;

        self.add_account_balance(&account_addr, &asset_info_a, provide_value)?;
        self.add_account_balance(&account_addr, &asset_info_b, provide_value)?;

        let asset_entry_a = AssetEntry::new(&ans_asset_a);
        let asset_entry_b = AssetEntry::new(&ans_asset_b);

        // Exceed slippage tolerance
        let exceed_slippage_result = self.dex_adapter.ans_action(
            self.dex.name(),
            DexAnsAction::ProvideLiquidity {
                assets: vec![
                    AnsAsset::new(asset_entry_a.clone(), provide_value),
                    AnsAsset::new(
                        asset_entry_b.clone(),
                        Uint128::new(provide_value).mul_floor(Decimal::percent(69)),
                    ),
                ],
                max_spread: Some(Decimal::percent(30)),
            },
            &new_account,
            self.abstr_deployment.name_service(),
        );
        assert!(exceed_slippage_result.is_err());

        // Exceed slippage tolerance reverse
        let exceed_slippage_result = self.dex_adapter.ans_action(
            self.dex.name(),
            DexAnsAction::ProvideLiquidity {
                assets: vec![
                    AnsAsset::new(
                        asset_entry_b,
                        Uint128::new(provide_value).mul_floor(Decimal::percent(69)),
                    ),
                    AnsAsset::new(asset_entry_a, provide_value),
                ],
                max_spread: Some(Decimal::percent(30)),
            },
            &new_account,
            self.abstr_deployment.name_service(),
        );
        assert!(exceed_slippage_result.is_err());

        Ok(())
    }

    pub fn test_withdraw_liquidity(
        &self,
        provide_value_a: Option<u128>,
        provide_value_b: Option<u128>,
        // Defaults to [asset_a, asset_b]
        resulting_assets: Option<Vec<AssetInfoUnchecked>>,
    ) -> anyhow::Result<()> {
        let (ans_asset_a, asset_info_a) = self.dex.asset_a();
        let (ans_asset_b, asset_info_b) = self.dex.asset_b();

        let new_account = self
            .abstr_deployment
            .account_builder()
            .install_adapter::<DexAdapter<Chain>>()
            .build()?;
        let account_addr = new_account.address()?;

        let provide_value_a = provide_value_a.unwrap_or(1_000_000_000u128);
        let provide_value_b = provide_value_b.unwrap_or(1_000_000_000u128);

        self.add_account_balance(&account_addr, &asset_info_a, provide_value_a)?;
        self.add_account_balance(&account_addr, &asset_info_b, provide_value_b)?;

        let asset_entry_a = AssetEntry::new(&ans_asset_a);
        let asset_entry_b = AssetEntry::new(&ans_asset_b);

        // provide to the pool
        self.dex_adapter.ans_action(
            self.dex.name(),
            DexAnsAction::ProvideLiquidity {
                assets: vec![
                    AnsAsset::new(asset_entry_a.clone(), provide_value_a),
                    AnsAsset::new(asset_entry_b.clone(), provide_value_b),
                ],
                max_spread: Some(Decimal::percent(30)),
            },
            &new_account,
            self.abstr_deployment.name_service(),
        )?;

        // Check everything sent and we have some lp
        let asset_a_balance = self.query_addr_balance(&account_addr, &asset_info_a)?;
        assert!(asset_a_balance.is_zero());

        let asset_b_balance = self.query_addr_balance(&account_addr, &asset_info_b)?;
        assert!(asset_b_balance.is_zero());

        let lp_balance = self.query_addr_balance(&account_addr, &self.lp_asset)?;
        assert!(!lp_balance.is_zero());

        let lp_asset_entry = self
            .lp_asset
            .resolve(self.abstr_deployment.name_service())
            .unwrap();
        // withdraw_liquidity
        self.dex_adapter.ans_action(
            self.dex.name(),
            DexAnsAction::WithdrawLiquidity {
                lp_token: AnsAsset::new(lp_asset_entry, lp_balance / Uint128::new(2)),
            },
            &new_account,
            self.abstr_deployment.name_service(),
        )?;

        // After withdrawing, we should get some tokens in return and some lp token left
        let lp_balance = self.query_addr_balance(&account_addr, &self.lp_asset)?;
        assert!(!lp_balance.is_zero());

        let resulting_assets = resulting_assets.unwrap_or(vec![asset_info_a, asset_info_b]);

        for asset_info in resulting_assets {
            let asset_balance = self.query_addr_balance(&account_addr, &asset_info)?;
            assert!(!asset_balance.is_zero());
        }

        Ok(())
    }

    pub fn test_queries(&self) -> anyhow::Result<()>
    where
        Chain: TxHandler<Sender = Addr>,
    {
        let (ans_asset_a, asset_info_a) = self.dex.asset_a();
        let (ans_asset_b, asset_info_b) = self.dex.asset_b();

        let new_account = self
            .abstr_deployment
            .account_builder()
            .install_adapter::<DexAdapter<Chain>>()
            .build()?;
        let account_addr = new_account.address()?;

        let swap_value = 1_000_000_000u128;

        // We can get fees
        let dex_fees_response: DexFeesResponse = self
            .dex_adapter
            .query(&crate::msg::QueryMsg::Module(DexQueryMsg::Fees {}))?;
        let dex_fee_recipient_balance_before_swap =
            self.query_addr_balance(&dex_fees_response.recipient, &asset_info_a)?;

        let offer_asset = AnsAsset::new(AssetEntry::new(&ans_asset_a), swap_value);
        let ask_asset = AssetEntry::new(&ans_asset_b);
        // simulate swap 1_000_000_000 asset_a to asset_b
        let simulate_response: SimulateSwapResponse =
            self.dex_adapter
                .query(&crate::msg::QueryMsg::Module(DexQueryMsg::SimulateSwap {
                    offer_asset: offer_asset.clone(),
                    ask_asset: ask_asset.clone(),
                    dex: self.dex.name(),
                }))?;
        // Generate swap 1_000_000_000 asset_a to asset_b
        let generate_messages: GenerateMessagesResponse = self.dex_adapter.query(
            &crate::msg::QueryMsg::Module(DexQueryMsg::GenerateMessages {
                message: WholeDexAction(
                    self.dex.name(),
                    DexAnsAction::Swap {
                        offer_asset,
                        ask_asset,
                        max_spread: None,
                        belief_price: None,
                    },
                )
                .resolve(self.abstr_deployment.name_service())?,
                addr_as_sender: account_addr.to_string(),
            }),
        )?;

        self.add_account_balance(&account_addr, &asset_info_a, swap_value)?;
        // Send every message
        let mut chain = self.abstr_deployment.environment();
        for message in generate_messages.messages {
            match message {
                CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                    chain.add_balance(&Addr::unchecked(to_address), amount)?;
                }
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr,
                    msg,
                    funds,
                }) => {
                    chain.call_as(&account_addr).execute(
                        // Need to deserialize it back, to serialize
                        &from_json::<serde_json::Value>(&msg).unwrap(),
                        &funds,
                        &Addr::unchecked(contract_addr),
                    )?;
                }
                _ => unimplemented!(),
            }
        }

        // Check it's swapped for somewhere between return_amount +- spread_amount
        let asset_b_balance = self.query_addr_balance(&account_addr, &asset_info_b)?;
        assert!(
            (simulate_response.return_amount - simulate_response.spread_amount
                ..=simulate_response.return_amount + simulate_response.spread_amount)
                .contains(&asset_b_balance)
        );

        // Check Dex fee recipient received his fees
        let dex_fee_recipient_balance_after_swap =
            self.query_addr_balance(&dex_fees_response.recipient, &asset_info_a)?;
        assert_eq!(
            dex_fee_recipient_balance_before_swap + simulate_response.usage_fee,
            dex_fee_recipient_balance_after_swap
        );
        Ok(())
    }

    fn add_account_balance(
        &self,
        account_addr: &Addr,
        asset: &AssetInfoUnchecked,
        amount: u128,
    ) -> anyhow::Result<()> {
        let mut chain = self.abstr_deployment.environment();

        match asset {
            cw_asset::AssetInfoBase::Native(denom) => {
                chain.add_balance(account_addr, coins(amount, denom))?;
            }
            cw_asset::AssetInfoBase::Cw20(addr) => {
                chain.execute(
                    &cw20::Cw20ExecuteMsg::Mint {
                        recipient: account_addr.to_string(),
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

    fn query_addr_balance(
        &self,
        account_addr: &Addr,
        asset: &AssetInfoUnchecked,
    ) -> anyhow::Result<Uint128> {
        let chain = self.abstr_deployment.environment();

        let balance = match asset {
            cw_asset::AssetInfoBase::Native(denom) => {
                chain
                    .bank_querier()
                    .balance(account_addr, Some(denom.to_owned()))
                    .unwrap()
                    .pop()
                    .unwrap()
                    .amount
            }
            cw_asset::AssetInfoBase::Cw20(addr) => {
                let balance: cw20::BalanceResponse = chain
                    .query(
                        &cw20::Cw20QueryMsg::Balance {
                            address: account_addr.to_string(),
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
