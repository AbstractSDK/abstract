use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Decimal;

/// The target query implemented by the lsd hub contract (or any other oracle contract).
#[cw_serde]
#[derive(QueryResponses)]
pub enum TargetQuery {
    #[returns(TargetValueResponse)]
    TargetValue {},
}

#[cw_serde]
pub struct TargetValueResponse {
    /// Current exchange rate between the LSD token and the underlying native token minus liquidity discount
    pub target_value: Decimal,
}
