use crate::AVAILABLE_CHAINS;
use crate::NIBIRU;
use abstract_dex_standard::Identify;
// Source https://github.com/cosmorama/wynddex/tree/v1.0.0
#[derive(Default)]
pub struct Nibiru {}

impl Identify for Nibiru {
    fn name(&self) -> &'static str {
        NIBIRU
    }
    fn is_available_on(&self, chain_name: &str) -> bool {
        AVAILABLE_CHAINS.contains(&chain_name)
    }
}

#[cfg(feature = "full_integration")]
use ::{
    abstract_dex_standard::{
        coins_in_assets, cw_approve_msgs, DexCommand, DexError, Fee, FeeOnInput, Return, Spread,
    },
    abstract_sdk::core::objects::PoolAddress,
    abstract_sdk::cw_helpers::wasm_smart_query,
    cosmwasm_std::{to_binary, wasm_execute, CosmosMsg, Decimal, Deps, Uint128, QueryRequest, Addr, Binary},
    // cw20::Cw20ExecuteMsg,
    cw_asset::{Asset, AssetInfo, AssetInfoBase},
    nibiru_std::proto::nibiru::spot::{QueryPoolResponse, MsgJoinPool, MsgExitPool, QuerySwapExactAmountInRequest, QuerySwapExactAmountInResponse},
};

#[cfg(feature = "full_integration")]
impl DexCommand<DexError> for Nibiru {
    fn swap(
        &self,
        _deps: Deps,
        proxy_addr: &Addr,
        pool_id: PoolAddress,
        offer_asset: Asset,
        ask_asset: AssetInfo,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pair_id = pool_id.expect_id()?;

        let swap_msg = nibiru_std::proto::nibiru::spot::MsgSwapAssets{
            sender: proxy_addr.to_string(),
            pool_id: pair_id,
            token_in: Some(cw_asset_to_coin(&offer_asset)?),      
            token_out_denom: cw_asset_to_denom(&ask_asset)?
        };

        let swap_msg = CosmosMsg::Stargate{
            type_url: "nibiru.spot.v1."
            value: Binary::from(swap_msg.to_any())
        };

        Ok(vec![swap_msg])
    }

    fn provide_liquidity(
        &self,
        deps: Deps,
        proxy_addr: &Addr,
        pool_id: PoolAddress,
        mut offer_assets: Vec<Asset>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pair_id = pool_id.expect_id()?;
        let mut msgs = vec![];

        // We know that (+)two assets were provided because it's a requirement to resolve the pool
        // We don't know if one of the asset amounts is 0, which would require a simulation and swap before providing liquidity
        if offer_assets.len() > 2 {
            return Err(DexError::TooManyAssets(2));
        } else if offer_assets.iter().any(|a| a.amount.is_zero()) {
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
                pool_id,
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

        let proto_assets = offer_assets
            .iter()
            .map(cw_asset_to_coin)
            .collect::<Result<Vec<_>, _>>()?;

        // execute msg
        let msg = nibiru_std::proto::nibiru::spot::MsgJoinPool{
            sender: "FAKE-SENDER    ".to_string(),
            pool_id: pair_id,
            tokens_in: proto_assets,      
            use_all_coins: true
        };

        msgs.push(msg);
        Ok(msgs)
    }

    fn provide_liquidity_symmetric(
        &self,
        deps: Deps,
        proxy_addr: &Addr,
        pool_id: PoolAddress,
        offer_asset: Asset,
        paired_assets: Vec<AssetInfo>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pair_id = pool_id.expected_id()?;

        if paired_assets.len() > 1 {
            return Err(DexError::TooManyAssets(2));
        }
        // Get pair info
        let pair_config: QueryPoolResponse = deps.querier.query(QueryRequest::Stargate())?;
        let other_asset = if pair_config.assets[0].info == offer_asset.info {
            let price =
                Decimal::from_ratio(pair_config.assets[1].amount, pair_config.assets[0].amount);
            let other_token_amount = price * offer_asset.amount;
            Asset {
                amount: other_token_amount,
                info: paired_assets[0].clone(),
            }
        } else if pair_config.assets[1].info == offer_asset.info {
            let price =
                Decimal::from_ratio(pair_config.assets[0].amount, pair_config.assets[1].amount);
            let other_token_amount = price * offer_asset.amount;
            Asset {
                amount: other_token_amount,
                info: paired_assets[0].clone(),
            }
        } else {
            return Err(DexError::ArgumentMismatch(
                offer_asset.to_string(),
                pair_config
                    .assets
                    .iter()
                    .map(|e| e.info.to_string())
                    .collect(),
            ));
        };

        let offer_assets = [offer_asset, other_asset];

        let coins = coins_in_assets(&offer_assets);

        // construct execute msg
        let proto_assets = offer_assets
            .iter()
            .map(cw_asset_to_coin)
            .collect::<Result<Vec<_>, _>>()?;

        let msg = MsgJoinPool {
            sender: proxy_addr,
            pool_id: pair_id,
            tokens_in: vec![proto_assets[0].clone(), proto_assets[1].clone()],
            use_all_coins: true
        };
        
        Ok(vec![msg])
    }

    fn withdraw_liquidity(
        &self,
        _deps: Deps,
        proxy_addr: &Addr,
        pool_id: PoolAddress,
        lp_token: Asset,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pair_id = pool_id.expected_id()?;
        let withdraw_message = MsgExitPool{ 
            sender: proxy_addr.to_string(),
            pool_id: pool_id,
            pool_shares: Some(cw_asset_to_coin(lp_token))
        };
        Ok(vec![withdraw_message])
    }

    fn simulate_swap(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        ask_asset: AssetInfo,
    ) -> Result<(Return, Spread, Fee, FeeOnInput), DexError> {
        let pair_id = pool_id.expected_id()?;

        let query = QuerySwapExactAmountInRequest{
            pool_id: pair_id,
            token_in: Some(cw_asset_to_coin(&offer_asset)),
            token_out_denom: ask_asset
        };

        // Do simulation
        let QuerySwapExactAmountInResponse {
            token_out,
            fee
        } = deps.querier.query(QueryRequest::Stargate())?;
        // commission paid in result asset
        Ok((token_out.unwrap().denom, Uint128::zero(), fee.unwrap().denom, false))
    }
}

#[cfg(feature = "full_integration")]
fn cw_asset_to_coin(asset: &Asset) -> Result<nibiru_std::proto::cosmos::base::v1beta1::Coin, DexError> {
    match &asset.info {
        AssetInfoBase::Native(denom) => Ok(nibiru_std::proto::cosmos::base::v1beta1::Coin {
            denom: denom.clone(),
            amount: asset.amount.to_string(),
        }),
        AssetInfoBase::Cw20(contract_addr) => Err(DexError::UnsupportedAssetType(asset.to_string())),
    }
}

#[cfg(feature = "full_integration")]
fn cw_asset_to_denom(asset: &AssetInfo) -> Result<String, DexError> {
    match &asset {
        AssetInfoBase::Native(denom) => Ok(denom.clone()),
        AssetInfoBase::Cw20(contract_addr) => Err(DexError::UnsupportedAssetType(asset.to_string())),
    }
}