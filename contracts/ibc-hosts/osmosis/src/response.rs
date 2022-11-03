use abstract_os::dex::OfferAsset;

/// Return the data field for each message
#[cosmwasm_schema::cw_serde]
pub struct SwapResponse {
    pub offer_asset: OfferAsset,
    pub amount_out: OfferAsset,
}
