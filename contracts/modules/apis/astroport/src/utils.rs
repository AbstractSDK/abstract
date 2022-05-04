use cosmwasm_std::{Addr, Deps, Uint128};

use pandora_os::modules::dapp_base::error::BaseDAppError;
use pandora_os::native::memory::item::Memory;
use pandora_os::queries::terraswap::query_asset_balance;

/// Checks if the given address has enough tokens with a given offer_id
pub fn has_sufficient_balance(
    deps: Deps,
    memory: &Memory,
    offer_id: &str,
    address: &Addr,
    required: Uint128,
) -> Result<(), BaseDAppError> {
    // Load asset
    let info = memory.query_asset(deps, offer_id)?;
    // Get balance and check
    if info.query_balance(&deps.querier, address)? < required {
        return Err(BaseDAppError::Broke {});
    }
    Ok(())
}
