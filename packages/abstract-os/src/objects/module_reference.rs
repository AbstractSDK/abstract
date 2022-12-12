use cosmwasm_std::Addr;

#[cosmwasm_schema::cw_serde]
#[non_exhaustive]
pub enum ModuleReference {
    /// Native Abstract Contracts
    Native(Addr),
    /// Installable extensions
    Extension(Addr),
    /// Installable apps
    App(u64),
    /// A stand-alone contract
    Standalone(u64),
}
