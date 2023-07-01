use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use wyndex::asset::AssetInfoValidated;

#[cw_serde]
pub struct Config {
    /// The LSD hub contract address to use for the conversion
    pub hub_contract: Addr,
    /// The address of the wyAsset to convert to
    pub token_contract: Addr,
    /// The denom of the base asset to convert from
    pub base_denom: String,
}

/// Temporary data used during the conversion process, stored to keep it between submessages
#[cw_serde]
pub struct TmpData {
    /// Address that owns all of the source lp and will own all of the converted stake
    pub lp_owner: Addr,
    /// Address of the pair contract that should receive the converted stake
    pub pair_contract_to: Addr,
    /// The unbonding period to stake the LP tokens to
    pub unbonding_period: u64,
    /// The assets of the pair contract we will convert to
    pub assets: Vec<AssetInfoValidated>,
}

/// Stores the config struct at the given key
pub const CONFIG: Item<Config> = Item::new("config");
pub const TMP_DATA: Item<TmpData> = Item::new("tmp_data");
