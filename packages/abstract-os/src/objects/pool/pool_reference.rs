use crate::objects::pool_id::{PoolAddress, UncheckedPoolAddress};
use crate::objects::unique_pool_id::UniquePoolId;
use crate::AbstractResult;
use cosmwasm_std::Api;

#[cosmwasm_schema::cw_serde]
pub struct PoolReference {
    pub unique_id: UniquePoolId,
    pub pool_address: PoolAddress,
}

impl PoolReference {
    pub fn new(unique_id: UniquePoolId, pool_address: PoolAddress) -> Self {
        Self {
            unique_id,
            pool_address,
        }
    }
}

/// Pool referenced with an unchecked pool ID
pub struct UncheckedPoolReference {
    pub unique_id: u64,
    pub pool_address: UncheckedPoolAddress,
}

impl UncheckedPoolReference {
    pub fn new(unique_id: u64, pool_address: UncheckedPoolAddress) -> Self {
        Self {
            unique_id,
            pool_address,
        }
    }

    pub fn check(&self, api: &dyn Api) -> AbstractResult<PoolReference> {
        let checked_pool_address = self.pool_address.check(api)?;

        Ok(PoolReference::new(
            UniquePoolId::new(self.unique_id),
            checked_pool_address,
        ))
    }
}
