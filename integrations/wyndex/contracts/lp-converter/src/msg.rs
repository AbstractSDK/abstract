use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    /// Address of the hub contract that will be used to convert the stake
    pub hub: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
