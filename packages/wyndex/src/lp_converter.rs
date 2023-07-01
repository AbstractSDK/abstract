use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub enum ExecuteMsg {
    /// Message sent by the staking contract, along with the freed LP tokens to initiate the conversion
    Convert {
        /// The address that will own the converted stake.
        /// The staking contract will put the sender of the `MigrateStake` message here.
        sender: String,
        /// How many LP tokens were freed by the staking contract
        amount: Uint128,
        /// The unbonding period to stake the target LP tokens to
        unbonding_period: u64,
        /// Address of the pair contract whose LP tokens should be converted
        pair_contract_from: String,
        /// Address of the pair contract that should receive the converted stake
        pair_contract_to: String,
    },
}
