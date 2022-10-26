use cosmwasm_std::Addr;

#[cosmwasm_schema::cw_serde]
pub enum ModuleReference {
    /// Native Abstract Contracts
    Native(Addr),
    /// Installable extensions
    Extension(Addr),
    /// Installable apps
    App(u64),
    /// Usable perks
    Perk(u64),
    /// usable services
    Service(u64),
}
