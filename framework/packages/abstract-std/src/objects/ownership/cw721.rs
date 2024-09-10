use cosmwasm_schema::cw_serde;

/// Expiration represents a point in time when some event happens.
/// It can compare with a BlockInfo and will return is_expired() == true
/// once the condition is hit (and for every block in the future)
// #[cw_serde]
// #[derive(Copy)]
// pub enum Expiration {
//     /// AtHeight will expire when `env.block.height` >= height
//     AtHeight(u64),
//     /// AtTime will expire when `env.block.time` >= time
//     AtTime(cosmwasm_std::Timestamp),
//     /// Never will never expire. Used to express the empty variant
//     Never {},
// }

// #[cw_serde]
// pub struct Approval {
//     /// Account that can transfer/send the token
//     pub spender: String,
//     /// When the Approval expires (maybe Expiration::never)
//     pub expires: Expiration,
// }

#[cw_serde]
pub struct OwnerOfResponse {
    /// Owner of the token
    pub owner: String,
    // Left out as an optimization.
    // /// If set this address is approved to transfer/send the token as well
    // pub approvals: Vec<Approval>,
}

#[cw_serde]
pub enum Cw721QueryMsg {
    /// Return the owner of the given token, error if token does not exist
    /// Return type: OwnerOfResponse
    OwnerOf {
        token_id: String,
        /// unset or false will filter out expired approvals, you must set to true to see them
        include_expired: Option<bool>,
    },
}
