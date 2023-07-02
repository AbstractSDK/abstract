use cosmwasm_schema::cw_serde;
use cosmwasm_std::Decimal;

const MAX_TOTAL_FEE_BPS: u16 = 10_000;
const MAX_PROTOCOL_FEE_BPS: u16 = 10_000;

#[cw_serde]
pub struct FeeConfig {
    /// The total fees (in bps) charged by a pair of this type.
    /// In relation to the returned amount of tokens
    pub total_fee_bps: u16,
    /// The amount of fees (in bps) collected by the protocol from this pair type.
    /// In relation to the total fee
    pub protocol_fee_bps: u16,
}

impl FeeConfig {
    /// This method is used to check fee bps.
    pub fn valid_fee_bps(&self) -> bool {
        self.total_fee_bps <= MAX_TOTAL_FEE_BPS && self.protocol_fee_bps <= MAX_PROTOCOL_FEE_BPS
    }

    pub fn total_fee_rate(&self) -> Decimal {
        Decimal::from_ratio(self.total_fee_bps, 10_000u128)
    }

    pub fn protocol_fee_rate(&self) -> Decimal {
        Decimal::from_ratio(self.protocol_fee_bps, 10_000u128)
    }
}
