use cosmwasm_std::{Addr, DepsMut, Empty, MessageInfo, Response, StdResult};
use cosmwasm_std::{Env, StdError, Storage};
use cw_asset::{AssetInfo, AssetInfoUnchecked};

use abstract_os::ans_host::state::*;
use abstract_os::ans_host::{AssetPair, ExecuteMsg};
use abstract_os::dex::DexName;
use abstract_os::objects::pool_id::{PoolId, UncheckedPoolId};
use abstract_os::objects::pool_metadata::PoolMetadata;
use abstract_os::objects::pool_reference::PoolReference;
use abstract_os::objects::{
    DexAssetPairing, UncheckedChannelEntry, UncheckedContractEntry, UniquePoolId,
};

use crate::contract::AnsHostResult;
use crate::error::AnsHostError;
use crate::error::AnsHostError::InvalidAssetCount;

const MIN_POOL_ASSETS: usize = 2;
const MAX_POOL_ASSETS: usize = 5;

/// Handles the common base execute messages
pub fn handle_message(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    message: ExecuteMsg,
) -> AnsHostResult {
    match message {
        ExecuteMsg::SetAdmin { admin } => set_admin(deps, info, admin),
        ExecuteMsg::UpdateContractAddresses { to_add, to_remove } => {
            update_contract_addresses(deps, info, to_add, to_remove)
        }
        ExecuteMsg::UpdateAssetAddresses { to_add, to_remove } => {
            update_asset_addresses(deps, info, to_add, to_remove)
        }
        ExecuteMsg::UpdateChannels { to_add, to_remove } => {
            update_channels(deps, info, to_add, to_remove)
        }
        ExecuteMsg::UpdateDexes { to_add, to_remove } => {
            update_dex_registry(deps, info, to_add, to_remove)
        }
        ExecuteMsg::UpdatePools { to_add, to_remove } => {
            update_pools(deps, info, to_add, to_remove)
        }
    }
}

//----------------------------------------------------------------------------------------
//  GOVERNANCE CONTROLLED SETTERS
//----------------------------------------------------------------------------------------

/// Adds, updates or removes provided addresses.
pub fn update_contract_addresses(
    deps: DepsMut,
    msg_info: MessageInfo,
    to_add: Vec<(UncheckedContractEntry, String)>,
    to_remove: Vec<UncheckedContractEntry>,
) -> AnsHostResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    for (key, new_address) in to_add.into_iter() {
        let key = key.check();
        // validate addr
        let addr = deps.as_ref().api.addr_validate(&new_address)?;

        // Update function for new or existing keys
        let insert = |_| -> StdResult<Addr> { Ok(addr) };
        CONTRACT_ADDRESSES.update(deps.storage, key, insert)?;
    }

    for key in to_remove {
        let key = key.check();
        CONTRACT_ADDRESSES.remove(deps.storage, key);
    }

    Ok(Response::new().add_attribute("action", "updated contract addresses"))
}

/// Adds, updates or removes provided addresses.
pub fn update_asset_addresses(
    deps: DepsMut,
    msg_info: MessageInfo,
    to_add: Vec<(String, AssetInfoUnchecked)>,
    to_remove: Vec<String>,
) -> AnsHostResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    for (name, new_asset) in to_add.into_iter() {
        // Update function for new or existing keys
        let api = deps.api;
        let insert = |_| -> StdResult<AssetInfo> {
            // use own check, cw_asset otherwise changes cases to lowercase
            new_asset.check(api, None)
        };
        ASSET_ADDRESSES.update(deps.storage, name.into(), insert)?;
    }

    for name in to_remove {
        ASSET_ADDRESSES.remove(deps.storage, name.into());
    }

    Ok(Response::new().add_attribute("action", "updated asset addresses"))
}

/// Adds, updates or removes provided addresses.
pub fn update_channels(
    deps: DepsMut,
    msg_info: MessageInfo,
    to_add: Vec<(UncheckedChannelEntry, String)>,
    to_remove: Vec<UncheckedChannelEntry>,
) -> AnsHostResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    for (key, new_channel) in to_add.into_iter() {
        let key = key.check();
        // Update function for new or existing keys
        let insert = |_| -> StdResult<String> { Ok(new_channel) };
        CHANNELS.update(deps.storage, key, insert)?;
    }

    for key in to_remove {
        let key = key.check();
        CHANNELS.remove(deps.storage, key);
    }

    Ok(Response::new().add_attribute("action", "updated contract addresses"))
}

/// Updates the dex registry with additions and removals
fn update_dex_registry(
    deps: DepsMut,
    info: MessageInfo,
    to_add: Vec<String>,
    to_remove: Vec<String>,
) -> AnsHostResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    if !to_add.is_empty() {
        let register_dex = |mut dexes: Vec<String>| -> StdResult<Vec<String>> {
            for dex in to_add {
                if !dexes.contains(&dex) {
                    dexes.push(dex.to_ascii_lowercase());
                }
            }
            Ok(dexes)
        };

        REGISTERED_DEXES.update(deps.storage, register_dex)?;
    }

    if !to_remove.is_empty() {
        let deregister_dex = |mut dexes: Vec<String>| -> StdResult<Vec<String>> {
            for dex in to_remove {
                dexes.retain(|x| x != &dex);
            }
            Ok(dexes)
        };
        REGISTERED_DEXES.update(deps.storage, deregister_dex)?;
    }

    Ok(Response::new().add_attribute("action", "update dex registry"))
}

fn update_pools(
    deps: DepsMut,
    info: MessageInfo,
    to_add: Vec<(UncheckedPoolId, PoolMetadata)>,
    to_remove: Vec<UniquePoolId>,
) -> AnsHostResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let original_unique_pool_id = CONFIG.load(deps.storage)?.next_unique_pool_id;
    let mut next_unique_pool_id = original_unique_pool_id;

    // only load dexes if necessary
    let registered_dexes = if to_add.is_empty() {
        vec![]
    } else {
        REGISTERED_DEXES.load(deps.storage)?
    };

    for (pool_id, mut pool_metadata) in to_add.into_iter() {
        let pool_id = pool_id.check(deps.api)?;

        let assets = &mut pool_metadata.assets;
        validate_pool_assets(deps.storage, assets)?;

        let dex = pool_metadata.dex.to_ascii_lowercase();
        if !registered_dexes.contains(&dex) {
            return Err(AnsHostError::UnregisteredDex { dex });
        }

        // Register each pair of assets as a pairing and link it to the pool id
        register_pool_pairings(deps.storage, next_unique_pool_id, pool_id, assets, &dex)?;

        POOL_METADATA.save(deps.storage, next_unique_pool_id, &pool_metadata)?;

        // Increment the unique pool id for the next pool
        next_unique_pool_id.increment();
    }

    for pool_id_to_remove in to_remove {
        // load the pool metadata
        let pool_metadata = POOL_METADATA.may_load(deps.storage, pool_id_to_remove)?;

        let pool_metadata = match pool_metadata {
            Some(pool_metadata) => pool_metadata,
            // THere is no existing metadata at that id, so we can skip it
            None => continue,
        };

        remove_pool_pairings(
            deps.storage,
            pool_id_to_remove,
            &pool_metadata.dex,
            &pool_metadata.assets,
        )?;

        // remove the pool metadata
        POOL_METADATA.remove(deps.storage, pool_id_to_remove);
    }

    // Only update the next pool id if necessary
    if next_unique_pool_id != original_unique_pool_id {
        CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
            config.next_unique_pool_id = next_unique_pool_id;
            Ok(config)
        })?;
    }

    Ok(Response::new().add_attribute("action", "updated pools"))
}

/// Execute an action on every asset pairing in the list of assets
/// Example: assets: [A, B, C] -> [A, B], [A, C], [B, C]
fn exec_on_asset_pairings<T, A, E>(assets: &[String], mut action: A) -> StdResult<()>
where
    A: FnMut(AssetPair) -> Result<T, E>,
    StdError: From<E>,
{
    for (i, asset_x) in assets.iter().enumerate() {
        for (j, asset_y) in assets.iter().enumerate() {
            // Skip self-pairings
            if i == j || asset_x == asset_y {
                continue;
            }
            let pair: AssetPair = (asset_x.clone(), asset_y.clone());
            action(pair)?;
        }
    }
    Ok(())
}

fn register_pool_pairings(
    storage: &mut dyn Storage,
    next_pool_id: UniquePoolId,
    pool_id: PoolId,
    assets: &[String],
    dex: &DexName,
) -> StdResult<()> {
    let register_pairing = |(asset_x, asset_y): AssetPair| {
        let key = DexAssetPairing::new(&asset_x, &asset_y, dex);

        let compound_pool_id = PoolReference {
            id: next_pool_id,
            pool_id: pool_id.clone(),
        };

        register_asset_pairing(storage, key, compound_pool_id)
    };

    exec_on_asset_pairings(assets, register_pairing)
}

/// Register an asset pairing to its pool id
/// We ignore any duplicates, which is why we don't check for them
fn register_asset_pairing(
    storage: &mut dyn Storage,
    pair: DexAssetPairing,
    compound_pool_id: PoolReference,
) -> Result<Vec<PoolReference>, StdError> {
    let insert = |ids: Option<Vec<PoolReference>>| -> StdResult<_> {
        let mut ids = ids.unwrap_or_default();

        ids.push(compound_pool_id);
        Ok(ids)
    };

    ASSET_PAIRINGS.update(storage, pair, insert)
}

/// Remove the unique_pool_id (which is getting removed) from the list of pool ids for each asset pairing
fn remove_pool_pairings(
    storage: &mut dyn Storage,
    pool_id_to_remove: UniquePoolId,
    dex: &DexName,
    assets: &[String],
) -> StdResult<()> {
    let remove_pairing_action = |(asset_x, asset_y): AssetPair| -> Result<(), StdError> {
        let key = DexAssetPairing::new(&asset_x, &asset_y, dex);

        // Action to remove the pool id from the list of pool ids for the asset pairing
        let remove_pool_id_action = |ids: Option<Vec<PoolReference>>| -> StdResult<_> {
            let mut ids = ids.unwrap_or_default();
            ids.retain(|id| id.id != pool_id_to_remove);
            Ok(ids)
        };

        let remaining_ids = ASSET_PAIRINGS.update(storage, key.clone(), remove_pool_id_action)?;

        // If there are no remaining pools, remove the asset pair from the map
        if remaining_ids.is_empty() {
            ASSET_PAIRINGS.remove(storage, key);
        }
        Ok(())
    };

    exec_on_asset_pairings(assets, remove_pairing_action)
}

/// unsure
fn validate_pool_assets(storage: &dyn Storage, assets: &mut [String]) -> Result<(), AnsHostError> {
    // convert all assets to lower
    for asset in assets.iter_mut() {
        *asset = asset.to_ascii_lowercase();
    }

    if assets.len() < MIN_POOL_ASSETS || assets.len() > MAX_POOL_ASSETS {
        return Err(InvalidAssetCount {
            min: MIN_POOL_ASSETS,
            max: MAX_POOL_ASSETS,
            provided: assets.len(),
        });
    }

    // Validate that each exists in the asset registry
    for asset in assets.iter() {
        if ASSET_ADDRESSES.may_load(storage, asset.into())?.is_none() {
            return Err(AnsHostError::UnregisteredAsset {
                asset: asset.clone(),
            });
        }
    }
    Ok(())
}

pub fn set_admin(deps: DepsMut, info: MessageInfo, admin: String) -> AnsHostResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let admin_addr = deps.api.addr_validate(&admin)?;
    let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
    ADMIN.execute_update_admin::<Empty, Empty>(deps, info, Some(admin_addr))?;
    Ok(Response::default()
        .add_attribute("previous admin", previous_admin)
        .add_attribute("admin", admin))
}

#[cfg(test)]
mod test {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{Addr, DepsMut};

    use abstract_os::ans_host::InstantiateMsg;

    use crate::contract;
    use crate::contract::{instantiate, AnsHostResult};
    use crate::error::AnsHostError;
    use abstract_testing::map_tester::CwMapTester;

    use super::*;
    use speculoos::prelude::*;

    type AnsHostTestResult = Result<(), AnsHostError>;

    const TEST_CREATOR: &str = "creator";

    fn mock_init(mut deps: DepsMut) -> AnsHostResult {
        let info = mock_info(TEST_CREATOR, &[]);

        instantiate(deps.branch(), mock_env(), info, InstantiateMsg {})
    }

    fn execute_helper(deps: DepsMut, msg: ExecuteMsg) -> AnsHostTestResult {
        contract::execute(deps, mock_env(), mock_info(TEST_CREATOR, &[]), msg)?;
        Ok(())
    }

    fn register_assets_helper(deps: DepsMut, assets: Vec<String>) -> AnsHostTestResult {
        let msg = ExecuteMsg::UpdateAssetAddresses {
            to_add: assets
                .iter()
                .map(|a| (a.clone(), AssetInfoUnchecked::native(a.clone())))
                .collect(),
            to_remove: vec![],
        };
        execute_helper(deps, msg)?;
        Ok(())
    }

    mod update_dexes {
        use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage};
        use cosmwasm_std::OwnedDeps;

        use super::*;

        #[test]
        fn register_dex() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let info = mock_info(TEST_CREATOR, &[]);
            let new_dex = "test_dex".to_string();

            let msg = ExecuteMsg::UpdateDexes {
                to_add: vec![new_dex.clone()],
                to_remove: vec![],
            };

            let _res = contract::execute(deps.as_mut(), mock_env(), info, msg)?;

            assert_expected_dexes(&deps, vec![new_dex]);

            Ok(())
        }

        /// Registering multiple dexes should work
        #[test]
        fn register_dex_twice() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let info = mock_info(TEST_CREATOR, &[]);
            let new_dex = "test_dex".to_string();

            let msg = ExecuteMsg::UpdateDexes {
                to_add: vec![new_dex.clone()],
                to_remove: vec![],
            };

            let _res = contract::execute(deps.as_mut(), mock_env(), info.clone(), msg.clone())?;
            let _res = contract::execute(deps.as_mut(), mock_env(), info, msg)?;

            assert_expected_dexes(&deps, vec![new_dex]);

            Ok(())
        }

        #[test]
        fn duplicate_in_msg() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let info = mock_info(TEST_CREATOR, &[]);
            let new_dex = "test_dex".to_string();

            let msg = ExecuteMsg::UpdateDexes {
                to_add: vec![new_dex.clone(), new_dex.clone()],
                to_remove: vec![],
            };

            let _res = contract::execute(deps.as_mut(), mock_env(), info, msg)?;

            // ONly one dex should be registered
            assert_expected_dexes(&deps, vec![new_dex]);

            Ok(())
        }

        #[test]
        fn register_and_deregister_dex_same_msg() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let info = mock_info(TEST_CREATOR, &[]);
            let new_dex = "test_dex".to_string();

            let msg = ExecuteMsg::UpdateDexes {
                to_add: vec![new_dex.clone()],
                to_remove: vec![new_dex],
            };

            let _res = contract::execute(deps.as_mut(), mock_env(), info, msg)?;

            assert_expected_dexes(&deps, vec![]);

            Ok(())
        }

        #[test]
        fn register_multiple_dexes() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let info = mock_info(TEST_CREATOR, &[]);
            let new_dexes = vec!["test_dex".to_string(), "test_dex_2".to_string()];

            let msg = ExecuteMsg::UpdateDexes {
                to_add: new_dexes.clone(),
                to_remove: vec![],
            };

            let _res = contract::execute(deps.as_mut(), mock_env(), info, msg)?;

            assert_expected_dexes(&deps, new_dexes);

            Ok(())
        }

        #[test]
        fn remove_nonexistent_dex() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let info = mock_info(TEST_CREATOR, &[]);
            let missing_dex = "test_dex".to_string();

            let msg = ExecuteMsg::UpdateDexes {
                to_add: vec![],
                to_remove: vec![missing_dex],
            };

            let _res = contract::execute(deps.as_mut(), mock_env(), info, msg)?;

            let expected_dexes: Vec<String> = vec![];

            assert_expected_dexes(&deps, expected_dexes);

            Ok(())
        }

        fn assert_expected_dexes(
            deps: &OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
            expected_dexes: Vec<String>,
        ) {
            let actual_dexes = REGISTERED_DEXES.load(&deps.storage).unwrap();

            assert_eq!(actual_dexes, expected_dexes);
        }
    }

    mod update_contract_addresses {

        use abstract_os::objects::ContractEntry;
        use abstract_testing::map_tester::CwMapTesterBuilder;

        use super::*;

        fn contract_entry(provider: &str, name: &str) -> UncheckedContractEntry {
            UncheckedContractEntry {
                protocol: provider.to_string(),
                contract: name.to_string(),
            }
        }

        fn contract_address_map_entry(
            provider: &str,
            name: &str,
            address: &str,
        ) -> (UncheckedContractEntry, String) {
            (contract_entry(provider, name), address.to_string())
        }

        fn mock_contract_map_entry() -> (UncheckedContractEntry, String) {
            contract_address_map_entry("test_provider", "test_contract", "test_address")
        }

        fn update_contract_addresses_msg_builder(
            to_add: Vec<(UncheckedContractEntry, String)>,
            to_remove: Vec<UncheckedContractEntry>,
        ) -> ExecuteMsg {
            ExecuteMsg::UpdateContractAddresses { to_add, to_remove }
        }

        fn from_checked_entry(
            (key, value): (ContractEntry, Addr),
        ) -> (UncheckedContractEntry, String) {
            (
                UncheckedContractEntry {
                    protocol: key.protocol,
                    contract: key.contract,
                },
                value.into(),
            )
        }

        fn setup_map_tester<'a>() -> CwMapTester<
            'a,
            ExecuteMsg,
            AnsHostError,
            ContractEntry,
            Addr,
            UncheckedContractEntry,
            String,
        > {
            let info = mock_info(TEST_CREATOR, &[]);

            let tester = CwMapTesterBuilder::default()
                .info(info)
                .map(CONTRACT_ADDRESSES)
                .execute(contract::execute)
                .msg_builder(update_contract_addresses_msg_builder)
                .mock_entry_builder(mock_contract_map_entry)
                .from_checked_entry(from_checked_entry)
                .build()
                .unwrap();

            tester
        }

        #[test]
        fn add_contract_address() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let mut map_tester = setup_map_tester();
            map_tester.test_add_one(&mut deps)
        }

        #[test]
        fn add_contract_address_twice() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let mut map_tester = setup_map_tester();
            map_tester.test_add_one_twice(&mut deps)
        }

        #[test]
        fn add_contract_address_twice_in_same_msg() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let mut map_tester = setup_map_tester();
            map_tester.test_add_two_same(&mut deps)
        }

        #[test]
        fn add_and_remove_contract_address_same_msg() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let mut map_tester = setup_map_tester();
            map_tester.test_add_and_remove_same(&mut deps)
        }

        #[test]
        fn remove_non_existent_contract_address() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let mut map_tester = setup_map_tester();
            map_tester.test_remove_nonexistent(&mut deps)
        }

        #[test]
        fn add_multiple_contract_addresses() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();
            let mut map_tester = setup_map_tester();

            let new_entry_1 =
                contract_address_map_entry("test_provider", "test_contract", "test_address");
            let new_entry_2 =
                contract_address_map_entry("test_provider_2", "test_contract_2", "test_address_2");
            let new_entry_3 =
                contract_address_map_entry("test_provider_3", "test_contract_3", "test_address_3");

            map_tester.test_update_auto_expect(
                &mut deps,
                (vec![new_entry_1, new_entry_2, new_entry_3], vec![]),
            )
        }

        #[test]
        fn add_multiple_contract_addresses_and_deregister_one() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();
            let mut map_tester = setup_map_tester();

            let _info = mock_info(TEST_CREATOR, &[]);
            let new_entry_1 =
                contract_address_map_entry("test_provider", "test_contract", "test_address");
            let new_entry_2 =
                contract_address_map_entry("test_provider_2", "test_contract_2", "test_address_2");

            // add 1 and 2
            map_tester.test_update_auto_expect(
                &mut deps,
                (vec![new_entry_1.clone(), new_entry_2.clone()], vec![]),
            )?;

            let new_entry_3 =
                contract_address_map_entry("test_provider_3", "test_contract_3", "test_address_3");

            // Add 3 and remove 1, leaving 2 and 3
            map_tester.test_update_with_expected(
                &mut deps,
                (vec![new_entry_3.clone()], vec![new_entry_1.0]),
                vec![new_entry_2, new_entry_3],
            )
        }

        #[test]
        fn invalid_address_rejected() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();
            let mut map_tester = setup_map_tester();

            let bad_entry =
                contract_address_map_entry("test_provider", "test_contract", "BAD_ADDRESS");

            let res = map_tester.execute_update(deps.as_mut(), (vec![bad_entry], vec![]));

            assert_that!(res).is_err();
            assert_that(&res.unwrap_err())
                .matches(|err| matches!(err, AnsHostError::Std(StdError::GenericErr { .. })));

            Ok(())
        }
    }

    mod update_asset_addresses {
        use super::*;
        use abstract_os::objects::AssetEntry;
        use abstract_testing::map_tester::CwMapTesterBuilder;
        use cw_asset::AssetInfoBase;

        fn unchecked_asset_map_entry(
            name: &str,
            info: AssetInfoUnchecked,
        ) -> (String, AssetInfoUnchecked) {
            (name.into(), info)
        }

        fn mock_asset_map_entry() -> (String, AssetInfoUnchecked) {
            let name = "test";
            let info = AssetInfoUnchecked::native("utest".to_string());

            unchecked_asset_map_entry(name, info)
        }

        fn update_asset_addresses_msg_builder(
            to_add: Vec<(String, AssetInfoUnchecked)>,
            to_remove: Vec<String>,
        ) -> ExecuteMsg {
            ExecuteMsg::UpdateAssetAddresses { to_add, to_remove }
        }

        fn from_checked_entry(
            (key, value): (AssetEntry, AssetInfo),
        ) -> (String, AssetInfoUnchecked) {
            (key.to_string(), value.into())
        }

        fn mock_unchecked_entries() -> (
            (String, AssetInfoUnchecked),
            (String, AssetInfoUnchecked),
            (String, AssetInfoUnchecked),
        ) {
            let new_entry_1 =
                unchecked_asset_map_entry("juno", AssetInfoBase::Native("ujuno".into()));
            let new_entry_2 =
                unchecked_asset_map_entry("osmo", AssetInfoBase::Native("uosmo".into()));
            let new_entry_3 =
                unchecked_asset_map_entry("sjuno", AssetInfoBase::Cw20("junoxxxxxxxxx".into()));
            (new_entry_1, new_entry_2, new_entry_3)
        }

        fn setup_map_tester<'a>() -> CwMapTester<
            'a,
            ExecuteMsg,
            AnsHostError,
            AssetEntry,
            AssetInfo,
            String,
            AssetInfoUnchecked,
        > {
            let info = mock_info(TEST_CREATOR, &[]);

            let tester = CwMapTesterBuilder::default()
                .info(info)
                .map(ASSET_ADDRESSES)
                .execute(contract::execute)
                .msg_builder(update_asset_addresses_msg_builder)
                .mock_entry_builder(mock_asset_map_entry)
                .from_checked_entry(from_checked_entry)
                .build()
                .unwrap();

            tester
        }

        #[test]
        fn add_asset_address() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let mut map_tester = setup_map_tester();
            map_tester.test_add_one(&mut deps)
        }

        #[test]
        fn add_asset_address_twice() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let mut map_tester = setup_map_tester();
            map_tester.test_add_one_twice(&mut deps)
        }

        #[test]
        fn add_asset_address_twice_in_same_msg() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let mut map_tester = setup_map_tester();
            map_tester.test_add_two_same(&mut deps)
        }

        #[test]
        fn add_and_remove_asset_address_same_msg() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let mut map_tester = setup_map_tester();
            map_tester.test_add_and_remove_same(&mut deps)
        }

        #[test]
        fn remove_non_existent_asset_address() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let mut map_tester = setup_map_tester();
            map_tester.test_remove_nonexistent(&mut deps)
        }

        #[test]
        fn add_multiple_asset_addresses() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();
            let mut map_tester = setup_map_tester();

            let (new_entry_1, new_entry_2, new_entry_3) = mock_unchecked_entries();

            map_tester.test_update_auto_expect(
                &mut deps,
                (vec![new_entry_1, new_entry_2, new_entry_3], vec![]),
            )
        }

        #[test]
        fn add_multiple_asset_addresses_and_deregister_one() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();
            let mut map_tester = setup_map_tester();

            let (new_entry_1, new_entry_2, _new_entry_3) = mock_unchecked_entries();

            // add 1 and 2
            map_tester.test_update_auto_expect(
                &mut deps,
                (vec![new_entry_1.clone(), new_entry_2.clone()], vec![]),
            )?;

            let new_entry_3 = unchecked_asset_map_entry("usd", AssetInfoBase::Cw20("uusd".into()));

            // Add 3 and remove 1, leaving 2 and 3
            map_tester.test_update_with_expected(
                &mut deps,
                (vec![new_entry_3.clone()], vec![new_entry_1.0]),
                vec![new_entry_2, new_entry_3],
            )
        }

        #[test]
        fn bad_asset_address_throws() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();
            let mut map_tester = setup_map_tester();

            let bad_asset_address =
                unchecked_asset_map_entry("BAD", AssetInfoUnchecked::Cw20("BAD".into()));

            let err = map_tester
                .execute_update(deps.as_mut(), (vec![bad_asset_address], vec![]))
                .unwrap_err();

            assert!(matches!(
                err,
                AnsHostError::Std(StdError::GenericErr { .. })
            ));

            assert!(err.to_string().contains("address not normalized"));

            Ok(())
        }
    }

    mod update_channels {
        use super::*;
        use abstract_os::objects::ChannelEntry;
        use abstract_testing::map_tester::CwMapTesterBuilder;

        type UncheckedChannelMapEntry = (UncheckedChannelEntry, String);

        fn update_channels_msg_builder(
            to_add: Vec<UncheckedChannelMapEntry>,
            to_remove: Vec<UncheckedChannelEntry>,
        ) -> ExecuteMsg {
            ExecuteMsg::UpdateChannels { to_add, to_remove }
        }

        fn from_checked_entry((key, value): (ChannelEntry, String)) -> UncheckedChannelMapEntry {
            (
                UncheckedChannelEntry {
                    connected_chain: key.clone().connected_chain,
                    protocol: key.protocol,
                },
                value,
            )
        }

        fn unchecked_channel_map_entry(
            chain: &str,
            protocol: &str,
            channel_id: &str,
        ) -> UncheckedChannelMapEntry {
            let channel_entry = UncheckedChannelEntry::new(chain, protocol);
            (channel_entry, channel_id.to_string())
        }

        fn mock_unchecked_channel_map_entry() -> UncheckedChannelMapEntry {
            unchecked_channel_map_entry("test_chain", "test_protocol", "test_channel_id")
        }

        fn mock_unchecked_channel_entries() -> (
            UncheckedChannelMapEntry,
            UncheckedChannelMapEntry,
            UncheckedChannelMapEntry,
        ) {
            let new_entry_1 =
                unchecked_channel_map_entry("test_provider_1", "test_contract_1", "test_address_1");
            let new_entry_2 =
                unchecked_channel_map_entry("test_provider_2", "test_contract_2", "test_address_2");
            let new_entry_3 =
                unchecked_channel_map_entry("test_provider_3", "test_contract_3", "test_address_3");
            (new_entry_1, new_entry_2, new_entry_3)
        }

        fn setup_map_tester<'a>() -> CwMapTester<
            'a,
            ExecuteMsg,
            AnsHostError,
            ChannelEntry,
            String,
            UncheckedChannelEntry,
            String,
        > {
            let info = mock_info(TEST_CREATOR, &[]);

            let tester = CwMapTesterBuilder::default()
                .info(info)
                .map(CHANNELS)
                .execute(contract::execute)
                .msg_builder(update_channels_msg_builder)
                .mock_entry_builder(mock_unchecked_channel_map_entry)
                .from_checked_entry(from_checked_entry)
                .build()
                .unwrap();

            tester
        }

        #[test]
        fn add_channel() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let mut map_tester = setup_map_tester();
            map_tester.test_add_one(&mut deps)
        }

        #[test]
        fn add_channel_twice() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let mut map_tester = setup_map_tester();
            map_tester.test_add_one_twice(&mut deps)
        }

        #[test]
        fn add_channel_twice_in_same_msg() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let mut map_tester = setup_map_tester();
            map_tester.test_add_two_same(&mut deps)
        }

        #[test]
        fn add_and_remove_channel_same_msg() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let mut map_tester = setup_map_tester();
            map_tester.test_add_and_remove_same(&mut deps)
        }

        #[test]
        fn remove_non_existent_channel() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let mut map_tester = setup_map_tester();
            map_tester.test_remove_nonexistent(&mut deps)
        }

        #[test]
        fn add_multiple_channels() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();
            let mut map_tester = setup_map_tester();

            let (new_entry_1, new_entry_2, new_entry_3) = mock_unchecked_channel_entries();

            map_tester.test_update_auto_expect(
                &mut deps,
                (vec![new_entry_1, new_entry_2, new_entry_3], vec![]),
            )
        }

        #[test]
        fn add_multiple_channels_and_deregister_one() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();
            let mut map_tester = setup_map_tester();

            let (new_entry_1, new_entry_2, _new_entry_3) = mock_unchecked_channel_entries();

            // add 1 and 2
            map_tester.test_update_auto_expect(
                &mut deps,
                (vec![new_entry_1.clone(), new_entry_2.clone()], vec![]),
            )?;

            let new_entry_3 =
                unchecked_channel_map_entry("test_provider_3", "test_contract_3", "test_address_3");

            // Add 3 and remove 1, leaving 2 and 3
            map_tester.test_update_with_expected(
                &mut deps,
                (vec![new_entry_3.clone()], vec![new_entry_1.0]),
                vec![new_entry_2, new_entry_3],
            )
        }

        #[test]
        fn upper_channel_entry_goes_lower() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();
            let mut map_tester = setup_map_tester();

            let upper_entry = unchecked_channel_map_entry("UP_CHAIN", "UP_PROTOCOL", "channel_id");

            map_tester.execute_update(deps.as_mut(), (vec![upper_entry], vec![]))?;

            let expected_entry =
                unchecked_channel_map_entry("up_chain", "up_protocol", "channel_id");
            map_tester.assert_expected_entries(&deps.storage, vec![expected_entry]);

            Ok(())
        }
    }

    mod update_pools {
        use super::*;
        use abstract_os::ans_host::{AssetPairingMapEntry, PoolMetadataMapEntry};
        use abstract_os::objects::PoolType;

        use cosmwasm_std::{Api, Order};
        use speculoos::assert_that;

        type UncheckedPoolMapEntry = (UncheckedPoolId, PoolMetadata);

        const INITIAL_UNIQUE_POOL_ID: u64 = 1;

        // Makes a stable
        fn pool_metadata(dex: &str, pool_type: PoolType, assets: Vec<String>) -> PoolMetadata {
            PoolMetadata {
                dex: dex.to_string(),
                pool_type,
                assets,
            }
        }

        fn _mock_pool_metadata() -> PoolMetadata {
            pool_metadata(
                "junoswap",
                PoolType::Weighted,
                vec!["juno".into(), "osmo".into()],
            )
        }

        fn unchecked_pool_map_entry(
            pool_contract_addr: &str,
            metadata: PoolMetadata,
        ) -> UncheckedPoolMapEntry {
            let pool_id = UncheckedPoolId::contract(pool_contract_addr);
            (pool_id, metadata)
        }

        fn _mock_unchecked_pool_map_entry() -> UncheckedPoolMapEntry {
            unchecked_pool_map_entry("junoxxxxx", _mock_pool_metadata())
        }

        fn build_update_msg(
            to_add: Vec<UncheckedPoolMapEntry>,
            to_remove: Vec<UniquePoolId>,
        ) -> ExecuteMsg {
            ExecuteMsg::UpdatePools { to_add, to_remove }
        }

        fn execute_update(
            deps: DepsMut,
            (to_add, to_remove): (Vec<UncheckedPoolMapEntry>, Vec<UniquePoolId>),
        ) -> AnsHostTestResult {
            let msg = build_update_msg(to_add, to_remove);
            execute_helper(deps, msg)?;
            Ok(())
        }

        fn register_dex(deps: DepsMut, dex: &str) -> AnsHostTestResult {
            let msg = ExecuteMsg::UpdateDexes {
                to_add: vec![dex.into()],
                to_remove: vec![],
            };
            execute_helper(deps, msg)?;
            Ok(())
        }

        fn load_pool_metadata(
            storage: &dyn Storage,
        ) -> Result<Vec<PoolMetadataMapEntry>, StdError> {
            POOL_METADATA
                .range(storage, None, None, Order::Ascending)
                .collect()
        }

        fn load_asset_pairings(
            storage: &dyn Storage,
        ) -> Result<Vec<AssetPairingMapEntry>, StdError> {
            ASSET_PAIRINGS
                .range(storage, None, None, Order::Ascending)
                .collect()
        }

        fn asset_pairing(
            api: &dyn Api,
            dex: &str,
            (asset_x, asset_y): (&str, &str),
            unchecked_pool_id: &UncheckedPoolId,
        ) -> Result<(DexAssetPairing, Vec<PoolReference>), StdError> {
            Ok((
                DexAssetPairing::new(asset_x, asset_y, dex),
                vec![PoolReference::new(
                    INITIAL_UNIQUE_POOL_ID.into(),
                    unchecked_pool_id.clone().check(api)?,
                )],
            ))
        }

        #[test]
        fn add_pool() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let dex = "junoswap";

            let pool_assets = vec!["juno".into(), "osmo".into()];
            let metadata = pool_metadata(dex.clone(), PoolType::Weighted, pool_assets.clone());

            // Register the assets in ANS
            register_assets_helper(deps.as_mut(), pool_assets)?;
            register_dex(deps.as_mut(), dex)?;

            let new_entry = unchecked_pool_map_entry("junoxxxx", metadata.clone());

            execute_update(deps.as_mut(), (vec![new_entry.clone()], vec![]))?;

            let expected_pools: Vec<PoolMetadataMapEntry> =
                vec![(INITIAL_UNIQUE_POOL_ID.into(), metadata)];
            let actual_pools: Result<Vec<PoolMetadataMapEntry>, _> =
                load_pool_metadata(&deps.storage);

            assert_that(&actual_pools?).is_equal_to(&expected_pools);

            let _pairing = DexAssetPairing::new("juno", "osmo", "junoswap");

            let (unchecked_pool_id, _) = new_entry;

            let expected_pairings = vec![
                asset_pairing(&deps.api, "junoswap", ("juno", "osmo"), &unchecked_pool_id)?,
                asset_pairing(&deps.api, "junoswap", ("osmo", "juno"), &unchecked_pool_id)?,
            ];
            let actual_pairings: Result<Vec<AssetPairingMapEntry>, _> =
                load_asset_pairings(&deps.storage);
            assert_that(&actual_pairings?).is_equal_to(&expected_pairings);

            Ok(())
        }

        #[test]
        fn add_five_asset_pool() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let dex = "junoswap";

            let pool_assets = vec![
                "juno".into(),
                "osmo".into(),
                "atom".into(),
                "uatom".into(),
                "uusd".into(),
            ];
            let metadata = pool_metadata(dex.clone(), PoolType::Weighted, pool_assets.clone());

            // Register the assets in ANS
            register_assets_helper(deps.as_mut(), pool_assets)?;
            register_dex(deps.as_mut(), dex)?;

            let new_entry = unchecked_pool_map_entry("junoxxxx", metadata.clone());

            execute_update(deps.as_mut(), (vec![new_entry.clone()], vec![]))?;

            let expected_pools: Vec<PoolMetadataMapEntry> =
                vec![(INITIAL_UNIQUE_POOL_ID.into(), metadata)];
            let actual_pools: Result<Vec<PoolMetadataMapEntry>, _> =
                load_pool_metadata(&deps.storage);

            assert_that(&actual_pools?).is_equal_to(&expected_pools);

            let _pairing = DexAssetPairing::new("juno", "osmo", "junoswap");

            let (unchecked_pool_id, _) = new_entry;

            // asset_count * (asset_count - 1)
            // Total pairs = 5 * (5 - 1) = 20
            let expected_pairing_count = 20;

            let actual_pairings = load_asset_pairings(&deps.storage)?;
            assert_that(&actual_pairings).has_length(expected_pairing_count);

            for (_pairing, ref_vec) in actual_pairings {
                assert_that(&ref_vec).has_length(1);
                // check the pool id is correct
                assert_that(&UncheckedPoolId::from(&ref_vec[0].pool_id))
                    .is_equal_to(&unchecked_pool_id);
            }

            Ok(())
        }

        #[test]
        fn add_pool_fails_without_registering_dex() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let unregistered_dex = "unregistered";

            let pool_assets = vec!["juno".into(), "osmo".into()];
            let metadata = pool_metadata(
                unregistered_dex.clone(),
                PoolType::Weighted,
                pool_assets.clone(),
            );
            // Register the assets in ANS
            register_assets_helper(deps.as_mut(), pool_assets)?;

            let entry = unchecked_pool_map_entry("junoxxxx", metadata);

            let res = execute_update(deps.as_mut(), (vec![entry], vec![]));

            assert_that(&res)
                .is_err()
                .is_equal_to(AnsHostError::UnregisteredDex {
                    dex: unregistered_dex.into(),
                });

            let actual_pools = load_pool_metadata(&deps.storage)?;
            assert_that(&actual_pools).is_empty();

            Ok(())
        }

        // THis test is weird because we remove the same one that is just created in this call
        #[test]
        fn add_and_remove_same_pool() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let dex = "junoswap";

            let pool_assets = vec!["juno".into(), "osmo".into()];
            let metadata = pool_metadata(dex.clone(), PoolType::Weighted, pool_assets.clone());

            // Register the assets in ANS
            register_assets_helper(deps.as_mut(), pool_assets)?;
            register_dex(deps.as_mut(), dex)?;

            let entry = unchecked_pool_map_entry("junoxxxx", metadata);

            execute_update(
                deps.as_mut(),
                (vec![entry], vec![INITIAL_UNIQUE_POOL_ID.into()]),
            )?;

            // metadata should be emtpy
            let actual_pools = load_pool_metadata(&deps.storage)?;
            assert_that(&actual_pools).is_empty();

            // all pairs should be empty
            let actual_pairs = load_asset_pairings(&deps.storage)?;
            assert_that(&actual_pairs).is_empty();

            Ok(())
        }

        #[test]
        fn remove_nonexistent_pool() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let res = execute_update(deps.as_mut(), (vec![], vec![INITIAL_UNIQUE_POOL_ID.into()]));

            assert_that(&res).is_ok();

            // metadata should be empty
            let actual_pools = load_pool_metadata(&deps.storage)?;
            assert_that(&actual_pools).is_empty();

            // all pairs should be empty
            let actual_pairs = load_asset_pairings(&deps.storage)?;
            assert_that(&actual_pairs).is_empty();

            Ok(())
        }

        #[test]
        fn unregistered_assets_fail() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let dex = "junoswap";

            let metadata = pool_metadata(
                dex.clone(),
                PoolType::Weighted,
                vec!["juno".into(), "osmo".into()],
            );

            register_dex(deps.as_mut(), dex)?;

            let entry = unchecked_pool_map_entry("junoxxxx", metadata);

            let res = execute_update(deps.as_mut(), (vec![entry], vec![]));

            assert_that(&res).is_err();

            assert_that(&res)
                .is_err()
                .is_equal_to(AnsHostError::UnregisteredAsset {
                    asset: "juno".to_string(),
                });

            Ok(())
        }
    }

    mod validate_pool_assets {
        use super::*;

        #[test]
        fn too_few() {
            let assets = &mut [];
            let deps = mock_dependencies();
            let result = validate_pool_assets(&deps.storage, assets).unwrap_err();
            assert_eq!(
                result,
                InvalidAssetCount {
                    min: MIN_POOL_ASSETS,
                    max: MAX_POOL_ASSETS,
                    provided: 0,
                }
            );

            let assets = &mut ["a".to_string()];
            let deps = mock_dependencies();
            let result = validate_pool_assets(&deps.storage, assets).unwrap_err();
            assert_eq!(
                result,
                InvalidAssetCount {
                    min: MIN_POOL_ASSETS,
                    max: MAX_POOL_ASSETS,
                    provided: 1,
                }
            );
        }

        #[test]
        fn unregistered() {
            let mut assets = vec!["a".to_string(), "b".to_string()];
            let deps = mock_dependencies();
            let res = validate_pool_assets(&deps.storage, &mut assets);

            assert_that(&res)
                .is_err()
                .is_equal_to(AnsHostError::UnregisteredAsset {
                    asset: "a".to_string(),
                });
        }

        #[test]
        fn valid_amounts() {
            let mut assets = vec!["a".to_string(), "b".to_string()];
            let mut deps = mock_dependencies();

            mock_init(deps.as_mut()).unwrap();
            register_assets_helper(deps.as_mut(), assets.clone()).unwrap();

            let res = validate_pool_assets(&deps.storage, &mut assets);

            assert_that(&res).is_ok();

            let mut assets: Vec<String> = vec!["a", "b", "c", "d", "e"]
                .iter()
                .map(|s| s.to_string())
                .collect();

            register_assets_helper(deps.as_mut(), assets.clone()).unwrap();
            let res = validate_pool_assets(&deps.storage, &mut assets);

            assert_that(&res).is_ok();
        }

        #[test]
        fn too_many() {
            let mut assets: Vec<String> = vec!["a", "b", "c", "d", "e", "f"]
                .iter()
                .map(|s| s.to_string())
                .collect();
            let deps = mock_dependencies();
            let result = validate_pool_assets(&deps.storage, &mut assets).unwrap_err();
            assert_eq!(
                result,
                InvalidAssetCount {
                    min: MIN_POOL_ASSETS,
                    max: MAX_POOL_ASSETS,
                    provided: 6,
                }
            );
        }
    }
}
