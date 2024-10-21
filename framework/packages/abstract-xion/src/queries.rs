use cosmwasm_std::{Binary, StdResult, Storage};

pub fn authenticator_by_id(storage: &dyn Storage, id: u8) -> StdResult<Binary> {
    cosmwasm_std::to_json_binary(&crate::state::AUTHENTICATORS.load(storage, id)?)
}

pub fn authenticator_ids(storage: &dyn Storage) -> StdResult<Binary> {
    cosmwasm_std::to_json_binary(
        &crate::state::AUTHENTICATORS
            .keys(storage, None, None, cosmwasm_std::Order::Ascending)
            .collect::<Result<Vec<_>, _>>()?,
    )
}
