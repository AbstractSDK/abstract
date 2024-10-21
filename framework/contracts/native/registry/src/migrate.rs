use abstract_std::{
    objects::{module_version::assert_cw_contract_upgrade, namespace::Namespace, AccountId},
    registry::{
        state::{NAMESPACES, REV_NAMESPACES},
        MigrateMsg,
    },
    REGISTRY,
};

use cosmwasm_std::{DepsMut, Env, Order, StdResult};
use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex};
use semver::Version;

use crate::contract::{VCResult, VcResponse, CONTRACT_VERSION};

pub struct NamespaceIndexes<'a> {
    pub account_id: MultiIndex<'a, AccountId, AccountId, &'a Namespace>,
}

impl<'a> IndexList<AccountId> for NamespaceIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<AccountId>> + '_> {
        let v: Vec<&dyn Index<AccountId>> = vec![&self.account_id];
        Box::new(v.into_iter())
    }
}

/// Primary index for namespaces.
pub const NAMESPACES_INFO: IndexedMap<&Namespace, AccountId, NamespaceIndexes> = IndexedMap::new(
    "nmspc",
    NamespaceIndexes {
        account_id: MultiIndex::new(|_pk, d| d.clone(), "nmspc", "nmspc_a"),
    },
);

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> VCResult {
    match msg {
        MigrateMsg::Instantiate(instantiate_msg) => abstract_sdk::cw_helpers::migrate_instantiate(
            deps,
            env,
            instantiate_msg,
            crate::contract::instantiate,
        ),
        MigrateMsg::Migrate {} => {
            let to_version: Version = CONTRACT_VERSION.parse()?;

            let namespaces_info = NAMESPACES_INFO
                .range(deps.storage, None, None, Order::Ascending)
                .collect::<StdResult<Vec<_>>>()?;
            for (namespace, account_id) in namespaces_info {
                NAMESPACES.save(deps.storage, &namespace, &account_id)?;
                REV_NAMESPACES.save(deps.storage, &account_id, &namespace)?;
            }
            assert_cw_contract_upgrade(deps.storage, REGISTRY, to_version)?;
            cw2::set_contract_version(deps.storage, REGISTRY, CONTRACT_VERSION)?;
            Ok(VcResponse::action("migrate"))
        }
    }
}
