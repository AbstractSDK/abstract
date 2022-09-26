use cosmwasm_std::Addr;

#[cosmwasm_schema::cw_serde]
pub enum ModuleReference {
    Extension(Addr),
    App(u64),
    Perk(u64),
    Service(u64),
}
