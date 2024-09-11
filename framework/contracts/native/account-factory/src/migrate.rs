use abstract_std::{
    account_factory::{
        state::{Config, CONFIG},
        MigrateMsg,
    },
    objects::module_version::assert_contract_upgrade,
    ACCOUNT_FACTORY,
};
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo};
use cw_storage_plus::Item;
use semver::Version;

use crate::contract::{
    instantiate, AccountFactoryResponse, AccountFactoryResult, CONTRACT_VERSION,
};

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> AccountFactoryResult {
    match msg {
        MigrateMsg::Instantiate(instantiate_msg) => {
            let contract_info = deps
                .querier
                .query_wasm_contract_info(&env.contract.address)?;
            // Only admin can call migrate on contract
            let sender = contract_info.admin.unwrap();
            let message_info = MessageInfo {
                sender,
                funds: vec![],
            };
            instantiate(deps, env, message_info, instantiate_msg)
        }
        MigrateMsg::Migrate {} => {
            let version: Version = CONTRACT_VERSION.parse().unwrap();

            assert_contract_upgrade(deps.storage, ACCOUNT_FACTORY, version)?;
            cw2::set_contract_version(deps.storage, ACCOUNT_FACTORY, CONTRACT_VERSION)?;

            Ok(AccountFactoryResponse::action("migrate"))
        }
    }
}
