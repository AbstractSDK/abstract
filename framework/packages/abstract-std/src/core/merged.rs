use super::*;

#[cosmwasm_schema::cw_serde]
pub struct InitMsg {
    pub manager: manager::InstantiateMsg,
    pub proxy: proxy::InstantiateMsg,
}

#[cosmwasm_schema::cw_serde]
pub enum ExecMsg {
    Manager(manager::ExecuteMsg),
    Proxy(proxy::ExecuteMsg),
}

#[cosmwasm_schema::cw_serde]
pub enum QueryMsg {
    Manager(manager::QueryMsg),
    Proxy(proxy::QueryMsg),
}
