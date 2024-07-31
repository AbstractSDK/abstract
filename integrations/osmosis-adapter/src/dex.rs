use abstract_dex_standard::Identify;
use abstract_sdk::feature_objects::VersionControlContract;
use cosmwasm_std::Addr;

use crate::{AVAILABLE_CHAINS, OSMOSIS};

#[derive(Default)]
pub struct Osmosis {
    pub version_control_contract: Option<VersionControlContract>,
    pub addr_as_sender: Option<Addr>,
}

impl Identify for Osmosis {
    fn is_available_on(&self, chain_name: &str) -> bool {
        AVAILABLE_CHAINS.contains(&chain_name)
    }
    fn name(&self) -> &'static str {
        OSMOSIS
    }
}

#[cfg(feature = "full_integration")]
use ::{
    abstract_dex_standard::{DexCommand, DexError, Fee, FeeOnInput, Return, Spread, SwapNode},
    abstract_sdk::{
        feature_objects::AnsHost, features::AbstractRegistryAccess, std::objects::PoolAddress,
        AbstractSdkError,
    },
    cosmwasm_std::{
        Coin, CosmosMsg, Decimal, Decimal256, Deps, StdError, StdResult, Uint128, Uint256,
    },
    cw_asset::{Asset, AssetInfo},
    osmosis_std::{
        types::osmosis::gamm::v1beta1::{MsgExitPool, MsgJoinPool, MsgSwapExactAmountIn},
        types::osmosis::poolmanager::v1beta1::{
            EstimateSwapExactAmountInRequest, PoolRequest, SwapAmountInRoute,
        },
        types::{cosmos::base::v1beta1::Coin as OsmoCoin, osmosis::gamm::v1beta1::Pool},
    },
};

#[cfg(feature = "full_integration")]
impl AbstractRegistryAccess for Osmosis {
    fn abstract_registry(
        &self,
        _: cosmwasm_std::Deps<'_>,
    ) -> std::result::Result<VersionControlContract, abstract_sdk::AbstractSdkError> {
        self.version_control_contract
            .clone()
            .ok_or(AbstractSdkError::generic_err(
                "version_control address is not set",
            ))
        // We need to get to the version control somehow (possible from Ans Host ?)
    }
}

#[cfg(feature = "full_integration")]
/// Osmosis app-chain dex implementation
impl DexCommand for Osmosis {
    fn fetch_data(
        &mut self,
        _deps: Deps,
        addr_as_sender: Addr,
        version_control_contract: VersionControlContract,
        _ans_host: AnsHost,
    ) -> Result<(), DexError> {
        self.version_control_contract = Some(version_control_contract);

        self.addr_as_sender = Some(addr_as_sender);
        Ok(())
    }

    fn swap(
        &self,
        _deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        ask_asset: AssetInfo,
        _belief_price: Option<Decimal>,
        _max_spread: Option<Decimal>,
    ) -> Result<Vec<cosmwasm_std::CosmosMsg>, DexError> {
        let pair_address = pool_id.expect_id()?;

        let token_out_denom = match ask_asset {
            AssetInfo::Native(denom) => Ok(denom),
            // TODO: cw20? on osmosis?
            _ => Err(DexError::UnsupportedAssetType(ask_asset.to_string())),
        }?;

        let routes: Vec<SwapAmountInRoute> = vec![SwapAmountInRoute {
            pool_id: pair_address.to_string().parse::<u64>().unwrap(),
            token_out_denom,
        }];

        let token_in = Coin::try_from(offer_asset)?;

        let swap_msg: CosmosMsg = MsgSwapExactAmountIn {
            sender: self
                .addr_as_sender
                .as_ref()
                .expect("no local proxy")
                .to_string(),
            routes,
            token_in: Some(token_in.into()),
            token_out_min_amount: Uint128::one().to_string(),
        }
        .into();

        Ok(vec![swap_msg])
    }

    fn swap_route(
        &self,
        _deps: Deps,
        swap_route: Vec<SwapNode<Addr>>,
        offer_asset: Asset,
        _belief_price: Option<Decimal>,
        _max_spread: Option<Decimal>,
    ) -> Result<Vec<cosmwasm_std::CosmosMsg>, DexError> {
        let routes = swap_route
            .into_iter()
            .map(|swap_node| {
                let pair_address = swap_node.pool_id.expect_id()?;
                let token_out_denom = match swap_node.ask_asset {
                    AssetInfo::Native(denom) => Ok(denom),
                    // TODO: cw20? on osmosis?
                    _ => Err(DexError::UnsupportedAssetType(
                        swap_node.ask_asset.to_string(),
                    )),
                }?;
                Ok(SwapAmountInRoute {
                    pool_id: pair_address,
                    token_out_denom,
                })
            })
            .collect::<Result<_, DexError>>()?;

        let token_in = Coin::try_from(offer_asset)?;

        let swap_msg: CosmosMsg = MsgSwapExactAmountIn {
            sender: self
                .addr_as_sender
                .as_ref()
                .expect("no local proxy")
                .to_string(),
            routes,
            token_in: Some(token_in.into()),
            token_out_min_amount: Uint128::one().to_string(),
        }
        .into();

        Ok(vec![swap_msg])
    }

    fn provide_liquidity(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        mut offer_assets: Vec<Asset>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<cosmwasm_std::CosmosMsg>, DexError> {
        let mut msgs = vec![];

        if offer_assets.len() > 2 {
            return Err(DexError::TooManyAssets(2));
        }

        if offer_assets.iter().any(|a| a.amount.is_zero()) {
            // find 0 asset
            let (index, non_zero_offer_asset) = offer_assets
                .iter()
                .enumerate()
                .find(|(_, a)| !a.amount.is_zero())
                .ok_or(DexError::TooFewAssets {})?;

            // the other asset in offer_assets is the one with amount zero
            let ask_asset = offer_assets.get((index + 1) % 2).unwrap().info.clone();

            // we want to offer half of the non-zero asset to swap into the ask asset
            let offer_asset = Asset::new(
                non_zero_offer_asset.info.clone(),
                non_zero_offer_asset
                    .amount
                    .checked_div(Uint128::from(2u128))
                    .unwrap(),
            );

            // simulate swap to get the amount of ask asset we can provide after swapping
            let simulated_received = self
                .simulate_swap(
                    deps,
                    pool_id.clone(),
                    offer_asset.clone(),
                    ask_asset.clone(),
                )?
                .0;
            let swap_msg = self.swap(
                deps,
                pool_id.clone(),
                offer_asset.clone(),
                ask_asset.clone(),
                None,
                max_spread,
            )?;
            // add swap msg
            msgs.extend(swap_msg);
            // update the offer assets for providing liquidity
            offer_assets = vec![offer_asset, Asset::new(ask_asset, simulated_received)];
        }
        let pool_id = pool_id.expect_id()?;

        let token_in_maxs: Vec<OsmoCoin> = {
            let mut tokens: Vec<OsmoCoin> = offer_assets
                .iter()
                .map(|asset| Coin::try_from(asset).unwrap().into())
                .collect();
            // Make sure they are sorted
            tokens.sort_by(|a, b| a.denom.cmp(&b.denom));
            tokens
        };

        let pool = query_pool_data(deps, pool_id)?;

        // check for symmetric pools
        if pool.pool_assets[0].weight != pool.pool_assets[1].weight {
            return Err(DexError::BalancerNotSupported(OSMOSIS.to_string()));
        }

        let pool_assets: Vec<OsmoCoin> = pool
            .pool_assets
            .into_iter()
            .map(|asset| asset.token.unwrap())
            .collect();

        let deposits: [Uint128; 2] = [
            token_in_maxs
                .iter()
                .find(|coin| coin.denom == pool_assets[0].denom)
                .map(|coin| coin.amount.parse::<Uint128>().unwrap())
                .expect("wrong asset provided"),
            token_in_maxs
                .iter()
                .find(|coin| coin.denom == pool_assets[1].denom)
                .map(|coin| coin.amount.parse::<Uint128>().unwrap())
                .expect("wrong asset provided"),
        ];

        assert_slippage_tolerance(&max_spread, &deposits, &pool_assets)?;

        let total_share = pool
            .total_shares
            .unwrap()
            .amount
            .parse::<Uint128>()
            .unwrap();

        let share_out_amount =
            compute_osmo_share_out_amount(&pool_assets, &deposits, total_share)?.to_string();

        let osmo_msg: CosmosMsg = MsgJoinPool {
            sender: self.addr_as_sender.as_ref().unwrap().to_string(),
            pool_id,
            share_out_amount,
            token_in_maxs,
        }
        .into();
        msgs.push(osmo_msg);

        Ok(msgs)
    }

    fn withdraw_liquidity(
        &self,
        _deps: Deps,
        pool_id: PoolAddress,
        lp_token: Asset,
    ) -> Result<Vec<cosmwasm_std::CosmosMsg>, DexError> {
        let pool_id = pool_id.expect_id()?;
        let osmo_msg: CosmosMsg = MsgExitPool {
            sender: self.addr_as_sender.as_ref().unwrap().to_string(),
            pool_id,
            share_in_amount: lp_token.amount.to_string(),
            token_out_mins: vec![], // This is fine! see: https://github.com/osmosis-labs/osmosis/blob/c51a248d67cd58e47587d6a955c3d765734eddd7/x/gamm/keeper/pool_service.go#L372
        }
        .into();

        Ok(vec![osmo_msg])
    }

    fn simulate_swap(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        ask_asset: AssetInfo,
    ) -> Result<(Return, Spread, Fee, FeeOnInput), DexError> {
        let pool_id = pool_id.expect_id()?;

        let routes: Vec<SwapAmountInRoute> = vec![SwapAmountInRoute {
            pool_id: pool_id.to_string().parse::<u64>().unwrap(),
            token_out_denom: match ask_asset {
                AssetInfo::Native(denom) => Ok(denom),
                _ => Err(DexError::UnsupportedAssetType(ask_asset.to_string())),
            }?,
        }];

        let token_in = Coin::try_from(offer_asset)?.to_string();

        #[allow(deprecated)]
        let swap_exact_amount_in_response = EstimateSwapExactAmountInRequest {
            pool_id: pool_id.to_string().parse::<u64>().unwrap(),
            token_in,
            routes,
        }
        .query(&deps.querier)
        .unwrap();

        Ok((
            swap_exact_amount_in_response
                .token_out_amount
                .parse::<Uint128>()
                .unwrap(),
            Uint128::zero(),
            Uint128::zero(),
            false,
        ))
    }
}

#[cfg(feature = "full_integration")]
fn query_pool_data(deps: Deps, pool_id: u64) -> StdResult<Pool> {
    let res = PoolRequest { pool_id }.query(&deps.querier).unwrap();

    let pool = Pool::try_from(res.pool.unwrap()).unwrap();
    Ok(pool)
}

#[cfg(feature = "full_integration")]
fn compute_osmo_share_out_amount(
    pool_assets: &[OsmoCoin],
    deposits: &[Uint128; 2],
    total_share: Uint128,
) -> StdResult<Uint128> {
    // ~ source: terraswap contract ~
    // min(1, 2)
    // 1. sqrt(deposit_0 * exchange_rate_0_to_1 * deposit_0) * (total_share / sqrt(pool_0 * pool_1))
    // == deposit_0 * total_share / pool_0
    // 2. sqrt(deposit_1 * exchange_rate_1_to_0 * deposit_1) * (total_share / sqrt(pool_1 * pool_1))
    // == deposit_1 * total_share / pool_1
    let share_amount_out = std::cmp::min(
        deposits[0].multiply_ratio(
            total_share,
            pool_assets[0].amount.parse::<Uint128>().unwrap(),
        ),
        deposits[1].multiply_ratio(
            total_share,
            pool_assets[1].amount.parse::<Uint128>().unwrap(),
        ),
    );

    Ok(share_amount_out)
}

#[cfg(feature = "full_integration")]
fn assert_slippage_tolerance(
    slippage_tolerance: &Option<Decimal>,
    deposits: &[Uint128; 2],
    pool_assets: &[OsmoCoin],
) -> Result<(), DexError> {
    if let Some(slippage_tolerance) = *slippage_tolerance {
        let slippage_tolerance: Decimal256 = slippage_tolerance.into();
        if slippage_tolerance > Decimal256::one() {
            return Err(DexError::Std(StdError::generic_err(
                "slippage_tolerance cannot bigger than 1",
            )));
        }

        let one_minus_slippage_tolerance = Decimal256::one() - slippage_tolerance;
        let deposits: [Uint256; 2] = [deposits[0].into(), deposits[1].into()];
        let pools: [Uint256; 2] = [
            pool_assets[0].amount.parse::<Uint256>().unwrap(),
            pool_assets[1].amount.parse::<Uint256>().unwrap(),
        ];

        // Ensure each prices are not dropped as much as slippage tolerance rate
        if Decimal256::from_ratio(deposits[0], deposits[1]) * one_minus_slippage_tolerance
            > Decimal256::from_ratio(pools[0], pools[1])
            || Decimal256::from_ratio(deposits[1], deposits[0]) * one_minus_slippage_tolerance
                > Decimal256::from_ratio(pools[1], pools[0])
        {
            return Err(DexError::MaxSlippageAssertion(
                slippage_tolerance.to_string(),
                OSMOSIS.to_owned(),
            ));
        }
    }

    Ok(())
}

#[cfg(feature = "full_integration")]
impl abstract_sdk::features::ModuleIdentification for Osmosis {
    fn module_id(&self) -> abstract_sdk::std::objects::module::ModuleId<'static> {
        abstract_dex_standard::DEX_ADAPTER_ID
    }
}
