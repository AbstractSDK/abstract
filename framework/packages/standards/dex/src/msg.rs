#![warn(missing_docs)]
//! # Dex Adapter API
// re-export response types
// re-export response types
use abstract_core::{
    adapter,
    objects::{
        ans_host::AnsHostError,
        fee::{Fee, UsageFee},
        pool_id::UncheckedPoolAddress,
        AnsAsset, AnsEntryConvertor, AssetEntry, DexAssetPairing, PoolAddress, PoolReference,
    },
    AbstractError, AbstractResult,
};
use abstract_sdk::{feature_objects::AnsHost, Resolve};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, CosmosMsg, Decimal, StdError, Uint128};
use cw_asset::{Asset, AssetBase, AssetInfoBase};

/// Max fee for the dex adapter actions
pub const MAX_FEE: Decimal = Decimal::percent(5);

/// The name of the dex to trade on.
pub type DexName = String;

/// The callback id for interacting with a dex over ibc
pub const IBC_DEX_PROVIDER_ID: &str = "IBC_DEX_ACTION";

/// Top-level Abstract Adapter execute message. This is the message that is passed to the `execute` entrypoint of the smart-contract.
pub type ExecuteMsg = adapter::ExecuteMsg<DexExecuteMsg>;
/// Top-level Abstract Adapter instantiate message. This is the message that is passed to the `instantiate` entrypoint of the smart-contract.
pub type InstantiateMsg = adapter::InstantiateMsg<DexInstantiateMsg>;
/// Top-level Abstract Adapter query message. This is the message that is passed to the `query` entrypoint of the smart-contract.
pub type QueryMsg = adapter::QueryMsg<DexQueryMsg>;

impl adapter::AdapterExecuteMsg for DexExecuteMsg {}
impl adapter::AdapterQueryMsg for DexQueryMsg {}

/// Response for simulating a swap.
#[cosmwasm_schema::cw_serde]
pub struct SimulateSwapResponse<A = AssetEntry> {
    /// The pool on which the swap was simulated
    pub pool: DexAssetPairing<A>,
    /// Amount you would receive when performing the swap.
    pub return_amount: Uint128,
    /// Spread in ask_asset for this swap
    pub spread_amount: Uint128,
    // LP/protocol fees could be withheld from either input or output so commission asset must be included.
    /// Commission charged for the swap
    pub commission: (A, Uint128),
    /// Adapter fee charged for the swap (paid in offer asset)
    pub usage_fee: Uint128,
}

/// Response from GenerateMsgs
#[cosmwasm_schema::cw_serde]
pub struct GenerateMessagesResponse {
    /// Messages generated for dex action
    pub messages: Vec<CosmosMsg>,
}

/// Response for Dex Fees
#[cosmwasm_schema::cw_serde]
pub struct DexFeesResponse {
    /// Fee for using swap action
    pub swap_fee: Fee,
    /// Address where all fees will go
    pub recipient: Addr,
}

/// Instantiation message for dex adapter
#[cosmwasm_schema::cw_serde]
pub struct DexInstantiateMsg {
    /// Fee charged on each swap.
    pub swap_fee: Decimal,
    /// Recipient account for fees.
    pub recipient_account: u32,
}

/// Dex Execute msg
#[cosmwasm_schema::cw_serde]
pub enum DexExecuteMsg {
    /// Update the fee
    UpdateFee {
        /// New fee to set
        swap_fee: Option<Decimal>,
        /// New recipient account for fees
        recipient_account: Option<u32>,
    },
    /// Action to perform on the DEX
    Action {
        /// The name of the dex to interact with
        dex: DexName,
        /// The action to perform
        action: DexAction,
    },
    /// Action to perform on the DEX
    RawAction {
        /// The name of the dex to interact with
        dex: DexName,
        /// The action to perform
        action: DexRawAction,
    },
}

/// Possible actions to perform on the DEX
#[cosmwasm_schema::cw_serde]
pub enum DexAction {
    /// Provide arbitrary liquidity
    ProvideLiquidity {
        // support complex pool types
        /// Assets to add
        assets: Vec<AnsAsset>,
        /// Max spread to accept, is a percentage represented as a decimal.
        max_spread: Option<Decimal>,
    },
    /// Provide liquidity equally between assets to a pool
    ProvideLiquiditySymmetric {
        /// The asset to offer
        offer_asset: AnsAsset,
        // support complex pool types
        /// Assets that are paired with the offered asset
        /// Should exclude the offer asset
        paired_assets: Vec<AssetEntry>,
    },
    /// Withdraw liquidity from a pool
    WithdrawLiquidity {
        /// The asset LP token that is provided.
        lp_token: AnsAsset,
    },
    /// Standard swap between one asset to another
    Swap {
        /// The asset to offer
        offer_asset: AnsAsset,
        /// The asset to receive
        ask_asset: AssetEntry,
        /// The percentage of spread compared to pre-swap price or belief price (if provided)
        max_spread: Option<Decimal>,
        /// The belief price when submitting the transaction.
        belief_price: Option<Decimal>,
    },
}

/// Possible raw actions to perform on the DEX
#[cosmwasm_schema::cw_serde]
pub enum DexRawAction {
    /// Provide arbitrary liquidity
    ProvideLiquidity {
        /// Pool to provide liquidity to
        pool: UncheckedPoolAddress,
        // support complex pool types
        /// Assets to add
        assets: Vec<AssetBase<String>>,
        /// Max spread to accept, is a percentage represented as a decimal.
        max_spread: Option<Decimal>,
    },
    /// Provide liquidity equally between assets to a pool
    ProvideLiquiditySymmetric {
        /// Pool to provide liquidity to
        pool: UncheckedPoolAddress,
        /// The asset to offer
        offer_asset: AssetBase<String>,
        // support complex pool types
        /// Assets that are paired with the offered asset
        /// Should exclude the offer asset
        paired_assets: Vec<AssetInfoBase<String>>,
    },
    /// Withdraw liquidity from a pool
    WithdrawLiquidity {
        /// Pool to withdraw liquidity from
        pool: UncheckedPoolAddress,
        /// The asset LP token that is provided.
        lp_token: AssetBase<String>,
    },
    /// Standard swap between one asset to another
    Swap {
        /// Pool used to swap
        pool: UncheckedPoolAddress,
        /// The asset to offer
        offer_asset: AssetBase<String>,
        /// The asset to receive
        ask_asset: AssetInfoBase<String>,
        /// The percentage of spread compared to pre-swap price or belief price (if provided)
        max_spread: Option<Decimal>,
        /// The belief price when submitting the transaction.
        belief_price: Option<Decimal>,
    },
}

/// Structure created to be able to resolve an action using ANS
pub struct WholeDexAction(pub DexName, pub DexAction);

/// Returns the first pool address to be able to swap given assets on the given dex
pub fn pool_address(
    dex: DexName,
    assets: (AssetEntry, AssetEntry),
    querier: &cosmwasm_std::QuerierWrapper,
    ans_host: &AnsHost,
) -> abstract_core::objects::ans_host::AnsHostResult<PoolAddress> {
    let dex_pair = DexAssetPairing::new(assets.0, assets.1, &dex);
    let mut pool_ref = ans_host.query_asset_pairing(querier, &dex_pair)?;
    // Currently takes the first pool found, but should be changed to take the best pool
    let found: PoolReference = pool_ref.pop().ok_or(AnsHostError::DexPairingNotFound {
        pairing: dex_pair,
        ans_host: ans_host.address.clone(),
    })?;
    Ok(found.pool_address)
}

impl Resolve for WholeDexAction {
    type Output = DexRawAction;

    fn resolve(
        &self,
        querier: &cosmwasm_std::QuerierWrapper,
        ans_host: &abstract_sdk::feature_objects::AnsHost,
    ) -> abstract_core::objects::ans_host::AnsHostResult<Self::Output> {
        match self.1.clone() {
            DexAction::ProvideLiquidity { assets, max_spread } => {
                let mut asset_names = assets
                    .iter()
                    .cloned()
                    .map(|a| a.name)
                    .take(2)
                    .collect::<Vec<_>>();
                let assets = assets.resolve(querier, ans_host)?;

                let pool_address = pool_address(
                    self.0.clone(),
                    (asset_names.swap_remove(0), asset_names.swap_remove(0)),
                    querier,
                    ans_host,
                )?;
                Ok(DexRawAction::ProvideLiquidity {
                    pool: pool_address.into(),
                    assets: assets.into_iter().map(Into::into).collect(),
                    max_spread,
                })
            }
            DexAction::ProvideLiquiditySymmetric {
                offer_asset,
                mut paired_assets,
            } => {
                let paired_asset_infos = paired_assets.resolve(querier, ans_host)?;
                let pool_address = pool_address(
                    self.0.clone(),
                    (paired_assets.swap_remove(0), offer_asset.name.clone()),
                    querier,
                    ans_host,
                )?;
                let offer_asset = offer_asset.resolve(querier, ans_host)?;
                Ok(DexRawAction::ProvideLiquiditySymmetric {
                    pool: pool_address.into(),
                    offer_asset: offer_asset.into(),
                    paired_assets: paired_asset_infos.into_iter().map(Into::into).collect(),
                })
            }
            DexAction::WithdrawLiquidity { lp_token } => {
                let lp_asset = lp_token.resolve(querier, ans_host)?;

                let lp_pairing: DexAssetPairing = AnsEntryConvertor::new(
                    AnsEntryConvertor::new(lp_token.name.clone()).lp_token()?,
                )
                .dex_asset_pairing()?;

                let mut pool_ids = lp_pairing.resolve(querier, ans_host)?;
                // TODO: when resolving if there are more than one, get the metadata and choose the one matching the assets
                if pool_ids.len() != 1 {
                    return Err(StdError::generic_err(format!(
                        "There are {} pairings for the given LP token",
                        pool_ids.len()
                    ))
                    .into());
                }

                let pool_address = pool_ids.pop().unwrap().pool_address;
                Ok(DexRawAction::WithdrawLiquidity {
                    pool: pool_address.into(),
                    lp_token: lp_asset.into(),
                })
            }
            DexAction::Swap {
                offer_asset,
                mut ask_asset,
                max_spread,
                belief_price,
            } => {
                let AnsAsset {
                    name: mut offer_asset,
                    amount: offer_amount,
                } = offer_asset.clone();
                offer_asset.format();
                ask_asset.format();

                let offer_asset_info = offer_asset.resolve(querier, ans_host)?;
                let ask_asset_info = ask_asset.resolve(querier, ans_host)?;

                let pool_address = pool_address(
                    self.0.clone(),
                    (offer_asset.clone(), ask_asset.clone()),
                    querier,
                    ans_host,
                )?;
                let offer_asset = Asset::new(offer_asset_info, offer_amount);

                Ok(DexRawAction::Swap {
                    pool: pool_address.into(),
                    offer_asset: offer_asset.into(),
                    ask_asset: ask_asset_info.into(),
                    max_spread,
                    belief_price,
                })
            }
        }
    }
}

/// Query messages for the dex adapter
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
pub enum DexQueryMsg {
    /// Simulate a swap between two assets
    /// Returns [`SimulateSwapResponse`]
    #[returns(SimulateSwapResponse)]
    SimulateSwap {
        /// The asset to offer
        offer_asset: AnsAsset,
        /// The asset to receive
        ask_asset: AssetEntry,
        /// Name of the dex to simulate the swap on
        dex: DexName,
    },
    /// Simulate a swap between two assets
    /// Returns [`SimulateSwapResponse`]
    #[returns(SimulateSwapResponse<AssetInfoBase<String>>)]
    SimulateSwapRaw {
        /// The asset to offer
        offer_asset: AssetBase<String>,
        /// The asset to receive
        ask_asset: AssetInfoBase<String>,
        /// Identifies of the pool to simulate the swap on.
        pool: UncheckedPoolAddress,
        /// Name of the dex to simulate the swap on
        dex: DexName,
    },
    /// Endpoint can be used by front-end to easily interact with contracts.
    /// Returns [`GenerateMessagesResponse`]
    #[returns(GenerateMessagesResponse)]
    GenerateMessages {
        /// Execute message to generate messages for
        message: DexExecuteMsg,
        /// Proxy Addr generate messages for
        proxy_addr: String,
    },
    /// Fee info for using the different dex actions
    #[returns(DexFeesResponse)]
    Fees {},
}

/// Fees for using the dex adapter
#[cosmwasm_schema::cw_serde]
pub struct DexFees {
    /// Fee for using swap action
    swap_fee: Fee,
    /// Address where all fees will go
    pub recipient: Addr,
}

impl DexFees {
    /// Create checked DexFees
    pub fn new(swap_fee_share: Decimal, recipient: Addr) -> AbstractResult<Self> {
        Self::check_fee_share(swap_fee_share)?;
        Ok(Self {
            swap_fee: Fee::new(swap_fee_share)?,
            recipient,
        })
    }

    /// Update swap share
    pub fn set_swap_fee_share(&mut self, new_swap_fee_share: Decimal) -> AbstractResult<()> {
        Self::check_fee_share(new_swap_fee_share)?;
        self.swap_fee = Fee::new(new_swap_fee_share)?;
        Ok(())
    }

    /// Get swap fee
    pub fn swap_fee(&self) -> Fee {
        self.swap_fee
    }

    /// Usage fee for swap
    pub fn swap_usage_fee(&self) -> AbstractResult<UsageFee> {
        UsageFee::new(self.swap_fee.share(), self.recipient.clone())
    }

    fn check_fee_share(fee: Decimal) -> AbstractResult<()> {
        if fee > MAX_FEE {
            return Err(AbstractError::Fee(format!(
                "fee share can't be bigger than {MAX_FEE}"
            )));
        }
        Ok(())
    }
}
