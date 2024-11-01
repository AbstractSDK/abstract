use cosmwasm_schema::cw_serde;

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
