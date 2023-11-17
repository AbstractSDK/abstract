use crate::{manager, proxy};

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub manager: crate::manager::InstantiateMsg,
    pub proxy: crate::proxy::InstantiateMsg,
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}

#[cosmwasm_schema::cw_serde]
#[serde(untagged)]
pub enum ExecuteMsg {
    Proxy(proxy::ExecuteMsg),
    Manager(manager::ExecuteMsg),
}

#[cosmwasm_schema::cw_serde]
#[serde(untagged)]
pub enum QueryMsg {
    Proxy(proxy::QueryMsg),
    Manager(manager::QueryMsg),
}
