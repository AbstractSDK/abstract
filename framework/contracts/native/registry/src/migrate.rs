use abstract_std::{
    objects::{
        module_version::assert_cw_contract_upgrade,
        namespace::{Namespace, ABSTRACT_NAMESPACE},
        storage_namespaces, AccountId, ABSTRACT_ACCOUNT_ID,
    },
    registry::{
        state::{CONFIG, NAMESPACES, REV_NAMESPACES},
        Config, MigrateMsg,
    },
    REGISTRY,
};

use cosmwasm_std::{from_json, DepsMut, Env, Order, StdResult};
use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex};
use semver::Version;

use crate::contract::{VCResult, VcResponse, CONTRACT_VERSION};

pub struct NamespaceIndexes<'a> {
    pub account_id: MultiIndex<'a, AccountId, AccountId, &'a Namespace>,
}

impl IndexList<AccountId> for NamespaceIndexes<'_> {
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

/// Contains configuration info of registry.
#[cosmwasm_schema::cw_serde]
pub struct ConfigV0_24 {
    pub security_disabled: bool,
    pub namespace_registration_fee: Option<cosmwasm_std::Coin>,
}

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
            for (namespace, account_id) in namespaces_info
                .into_iter()
                // Make sure abstract included
                .chain(std::iter::once((
                    Namespace::new(ABSTRACT_NAMESPACE)?,
                    ABSTRACT_ACCOUNT_ID,
                )))
            {
                NAMESPACES.save(deps.storage, &namespace, &account_id)?;
                REV_NAMESPACES.save(deps.storage, &account_id, &namespace)?;
            }
            // Migrate from 0_24 config
            let cfg = deps
                .storage
                .get(storage_namespaces::CONFIG_STORAGE_KEY.as_bytes())
                .unwrap();
            if let Ok(ConfigV0_24 {
                security_disabled,
                namespace_registration_fee,
            }) = from_json(cfg)
            {
                CONFIG.save(
                    deps.storage,
                    &Config {
                        security_enabled: !security_disabled,
                        namespace_registration_fee,
                    },
                )?;
            }
            assert_cw_contract_upgrade(deps.storage, REGISTRY, to_version)?;
            cw2::set_contract_version(deps.storage, REGISTRY, CONTRACT_VERSION)?;
            Ok(VcResponse::action("migrate"))
        }
    }
}
