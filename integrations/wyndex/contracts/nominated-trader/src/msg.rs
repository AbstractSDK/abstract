use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal, Uint128};
use wyndex::asset::{Asset, AssetInfo};

#[cw_serde]
pub struct InstantiateMsg {
    /// Address that's allowed to change contract parameters
    pub owner: String,
    /// Address that's allowed to perform swaps and convert fee tokens to Wynd as needed
    pub nominated_trader: String,
    /// Address specified to receive any payouts, usually distinct from the nominated_trader address
    pub beneficiary: String,
    /// The WYND token contract address
    pub token_contract: AssetInfo,
    /// The Wyndex factory contract address
    pub dex_factory_contract: String,
    /// The maximum spread used when swapping fee tokens to WYND
    pub max_spread: Option<Decimal>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Collects and swaps fee tokens to WYND after sending the Swap msgs as SubMsgs
    /// This call is restricted to the currently nominated trader which can perform fee collections and swaps
    Collect {
        /// The nominated assets to swap to WYND
        assets: Vec<AssetWithLimit>,
    },
    /// Add or remove route definitions for tokens used to swap specific fee tokens to WYND
    /// (effectively declaring the hops to be taken for a swap route)
    /// Only the `owner` can call this Execute action
    UpdateRoutes {
        add: Option<Vec<(AssetInfo, AssetInfo)>>,
        remove: Option<Vec<AssetInfo>>,
    },
    /// Swap fee tokens via hop assets
    /// Only the collector contract itself can call this
    /// No limit is specified for each asset and a maximum route depth is also exposed
    SwapHopAssets { assets: Vec<AssetInfo>, depth: u64 },
    /// Allows the owner to spend the contract's WYND balance. The trader contract will not be able to spend the WYND but can trade other assets to it.
    Transfer { recipient: String, amount: Uint128 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns information about the nominated trader configs
    #[returns(ConfigResponse)]
    Config {},
    /// Returns the balance for each asset in the specified input parameters
    #[returns(BalancesResponse)]
    Balances { assets: Vec<AssetInfo> },
    #[returns(RoutesResponse)]
    Routes {},
}

/// A custom struct that holds contract parameters and is used to retrieve them.
#[cw_serde]
pub struct ConfigResponse {
    /// Address that's allowed to change contract parameters
    pub owner: String,
    /// Address that's allowed to perform swaps and convert fee tokens to Wynd as needed
    pub nominated_trader: String,
    /// Address specified to receive any payouts usually distinct from the nominated_trader address
    pub beneficiary: String,
    /// The WYND token contract address
    pub token_contract: String,
    /// The Wyndex factory contract address
    pub dex_factory_contract: String,
    /// The maximum spread used when swapping fee tokens to WYND
    pub max_spread: Decimal,
}

/// A custom struct used to return multiple asset balances.
#[cw_serde]
pub struct BalancesResponse {
    pub balances: Vec<Asset>,
}

/// A custom struct used to return multiple asset balances.
#[cw_serde]
pub struct RoutesResponse {
    pub routes: Vec<(String, String)>,
}

/// This structure describes a migration message.
#[cw_serde]
pub struct MigrateMsg {}

/// This struct holds parameters to help with swapping a specific amount of a fee token to WYND.
#[cw_serde]
pub struct AssetWithLimit {
    /// Information about the fee token to swap
    pub info: AssetInfo,
    /// The amount of tokens to swap
    pub limit: Option<Uint128>,
}
