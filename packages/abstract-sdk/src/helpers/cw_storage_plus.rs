use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::{Map, PrimaryKey};
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Load a batch of values by their keys from a [`Map`].
pub fn load_many<'a, K, V>(
    map: Map<'a, K, V>,
    storage: &dyn Storage,
    keys: Vec<K>,
) -> StdResult<Vec<(K, V)>>
where
    K: PrimaryKey<'a>,
    V: DeserializeOwned + Serialize,
{
    let mut res: Vec<(K, V)> = vec![];

    for key in keys.into_iter() {
        let value = map.load(storage, key.clone())?;
        res.push((key, value));
    }

    Ok(res)
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, MockApi, MockQuerier, MockStorage};
    use cosmwasm_std::OwnedDeps;
    use speculoos::prelude::*;

    const TEST_MAP: Map<String, String> = Map::new("test_map");

    const EXISTING_KEYS: [&str; 4] = ["a", "b", "c", "d"];

    fn setup() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies();

        for key in EXISTING_KEYS.iter() {
            let string_key = key.to_string();
            TEST_MAP
                .save(deps.as_mut().storage, string_key.clone(), &string_key)
                .unwrap();
        }

        deps
    }

    #[test]
    fn load_many_works() {
        let deps = setup();
        let keys = EXISTING_KEYS
            .iter()
            .map(|key| key.to_string())
            .collect::<Vec<String>>();
        let res = load_many(TEST_MAP, deps.as_ref().storage, keys).unwrap();

        assert_that!(res).has_length(EXISTING_KEYS.len());

        for (key, value) in res.into_iter() {
            assert_that!(key).is_equal_to(value);
        }
    }

    #[test]
    fn load_many_with_not_existing() {
        let deps = setup();
        let with_non_existing = EXISTING_KEYS
            .iter()
            .map(|key| key.to_string())
            .chain(vec!["e".to_string()])
            .collect::<Vec<String>>();
        let res = load_many(TEST_MAP, deps.as_ref().storage, with_non_existing);

        assert_that!(res).is_err();
    }
}
