use cosmwasm_std::Addr;

/// Store the Version Control contract.
/// Implements [`AbstractRegistryAccess`]
#[cosmwasm_schema::cw_serde]
pub struct VersionControlContract {
    /// Address of the version control contract
    pub address: Addr,
}

impl VersionControlContract {
    /// Construct a new version control feature object.
    pub fn new(address: Addr) -> Self {
        Self { address }
    }
}
