use abstract_oracle_adapter::msg::Seconds;
use abstract_standalone::std::standalone::StandaloneInstantiateMsg;
use cosmwasm_std::Empty;

/// Standalone instantiate message
#[cosmwasm_schema::cw_serde]
pub struct MyStandaloneInstantiateMsg {
    pub base: StandaloneInstantiateMsg,
    pub denom_whitelist: Vec<String>,
    /// Max price age
    pub max_age: Seconds,
}

/// Standalone execute messages
pub type MyStandaloneExecuteMsg = cw_vault_standard::VaultStandardExecuteMsg<Empty>;

#[cosmwasm_schema::cw_serde]
pub struct MyStandaloneMigrateMsg {}

/// Standalone query messages
pub type MyStandaloneQueryMsg = cw_vault_standard::VaultStandardQueryMsg<Empty>;

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    /// The code ID of the cw-ica-controller contract.
    pub ica_controller_code_id: u64,
}

#[cosmwasm_schema::cw_serde]
pub struct ICACountResponse {
    pub count: u64,
}
