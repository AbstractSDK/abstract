use cosmwasm_std::{Addr, Deps, Uint128};
use dao_os::memory::item::Memory;
use dao_os::query::terraswap::query_asset_balance;
use dao_os::treasury::dapp_base::error::BaseDAppError;

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
    if query_asset_balance(deps, &info, address.clone())? < required {
        return Err(BaseDAppError::Broke {});
    }
    Ok(())
}
