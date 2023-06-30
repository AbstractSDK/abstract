use crate::objects::{pool_id::PoolAddress, unique_pool_id::UniquePoolId};

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
