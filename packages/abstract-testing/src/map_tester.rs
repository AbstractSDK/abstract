use crate::MockDeps;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::Order;
use cosmwasm_std::{DepsMut, MessageInfo, Response};
use cosmwasm_std::{Env, Storage};
use cw_storage_plus::{KeyDeserialize, Map, PrimaryKey};
use derive_builder::Builder;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json;
use serde_json::json;
use speculoos::prelude::*;
use std::fmt::Debug;

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct CwMapTester<'a, ExecMsg, TError, K, V, UncheckedK, UncheckedV>
where
    K: PrimaryKey<'a> + KeyDeserialize + Debug,
    K::Output: 'static,
{
    info: MessageInfo,
    map: Map<'a, K, V>,
    execute:
        fn(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecMsg) -> Result<Response, TError>,
    msg_builder: fn(to_add: Vec<(UncheckedK, UncheckedV)>, to_remove: Vec<UncheckedK>) -> ExecMsg,
    mock_entry_builder: fn() -> (UncheckedK, UncheckedV),
    from_checked_entry: fn((K::Output, V)) -> (UncheckedK, UncheckedV),
}

/// Sort the expected entries by *key*
fn sort_expected<K, V>(expected: &mut [(K, V)])
where
    K: Clone + PartialEq + Debug + Serialize,
    V: Clone + PartialEq + Debug,
{
    expected.sort_by(|a, b| json!(a.0).to_string().cmp(&json!(&b.0).to_string()));
}

#[allow(clippy::ptr_arg)]
pub fn determine_expected<K, V>(to_add: &Vec<(K, V)>, to_remove: &[K]) -> Vec<(K, V)>
where
    K: Clone + PartialEq + Debug + Serialize,
    V: Clone + PartialEq + Debug,
{
    let mut expected = to_add.clone();
    expected.retain(|(k, _)| !to_remove.contains(k));
    sort_expected(&mut expected);
    expected.dedup();
    expected
}

impl<'a, ExecMsg, TError, K, V, UncheckedK, UncheckedV>
    CwMapTester<'a, ExecMsg, TError, K, V, UncheckedK, UncheckedV>
where
    V: Serialize + DeserializeOwned + Clone + Debug,
    K: PrimaryKey<'a> + KeyDeserialize + Debug,
    (<K as KeyDeserialize>::Output, V): PartialEq<(K, V)>,
    K::Output: 'static,
    UncheckedK: Clone + PartialEq + Debug + Serialize,
    UncheckedV: Clone + PartialEq + Debug,
    <K as KeyDeserialize>::Output: Debug,
{
    pub fn new(
        info: MessageInfo,
        map: Map<'a, K, V>,
        execute: fn(
            deps: DepsMut,
            env: Env,
            info: MessageInfo,
            msg: ExecMsg,
        ) -> Result<Response, TError>,
        msg_builder: fn(
            to_add: Vec<(UncheckedK, UncheckedV)>,
            to_remove: Vec<UncheckedK>,
        ) -> ExecMsg,
        mock_entry_builder: fn() -> (UncheckedK, UncheckedV),
        from_checked_entry: fn((K::Output, V)) -> (UncheckedK, UncheckedV),
    ) -> Self {
        Self {
            info,
            map,
            execute,
            msg_builder,
            mock_entry_builder,
            from_checked_entry,
        }
    }

    pub fn msg_builder(
        &self,
        to_add: Vec<(UncheckedK, UncheckedV)>,
        to_remove: Vec<UncheckedK>,
    ) -> ExecMsg {
        (self.msg_builder)(to_add, to_remove)
    }

    fn mock_entry_builder(&self) -> (UncheckedK, UncheckedV) {
        (self.mock_entry_builder)()
    }

    /// Execute the msg with the mock env
    pub fn execute(&mut self, deps: DepsMut, msg: ExecMsg) -> Result<(), TError> {
        (self.execute)(deps, mock_env(), self.info.clone(), msg)?;
        Ok(())
    }

    pub fn execute_update(
        &mut self,
        deps: DepsMut,
        (to_add, to_remove): (Vec<(UncheckedK, UncheckedV)>, Vec<UncheckedK>),
    ) -> Result<(), TError> {
        let msg = self.msg_builder(to_add, to_remove);
        self.execute(deps, msg)
    }

    #[allow(clippy::wrong_self_convention)]
    fn from_checked_entry(&self, entry: (K::Output, V)) -> (UncheckedK, UncheckedV) {
        (self.from_checked_entry)(entry)
    }

    pub fn assert_expected_entries(
        &self,
        storage: &'_ dyn Storage,
        expected: Vec<(UncheckedK, UncheckedV)>,
    ) {
        let res: Result<Vec<(K::Output, V)>, _> = self
            .map
            .range(storage, None, None, Order::Ascending)
            .collect();

        let actual = res
            .unwrap()
            .into_iter()
            .map(|(k, v)| self.from_checked_entry((k, v)))
            .collect::<Vec<_>>();

        // Sort, like map entries
        let mut expected = expected;
        sort_expected(&mut expected);

        assert_that!(actual).is_equal_to(expected)
    }

    pub fn test_add_one(&mut self, deps: &mut MockDeps) -> Result<(), TError> {
        let entry = self.mock_entry_builder();

        let to_add: Vec<(UncheckedK, UncheckedV)> = vec![entry];
        let to_remove: Vec<UncheckedK> = vec![];
        let msg = self.msg_builder(to_add.clone(), to_remove.clone());

        let expected = determine_expected(&to_add, &to_remove);

        self.execute(deps.as_mut(), msg)?;

        self.assert_expected_entries(&deps.storage, expected);

        Ok(())
    }

    pub fn test_add_one_twice(&mut self, deps: &mut MockDeps) -> Result<(), TError> {
        self.test_add_one(deps)?;
        self.test_add_one(deps)
    }

    pub fn test_add_two_same(&mut self, deps: &mut MockDeps) -> Result<(), TError> {
        let entry = self.mock_entry_builder();

        let to_add: Vec<(UncheckedK, UncheckedV)> = vec![entry.clone(), entry];
        let to_remove: Vec<UncheckedK> = vec![];
        let msg = self.msg_builder(to_add.clone(), to_remove.clone());

        let expected: Vec<(UncheckedK, UncheckedV)> = determine_expected(&to_add, &to_remove);

        self.execute(deps.as_mut(), msg)?;

        self.assert_expected_entries(&deps.storage, expected);

        Ok(())
    }

    pub fn test_add_and_remove_same(&mut self, deps: &mut MockDeps) -> Result<(), TError> {
        let entry = self.mock_entry_builder();

        let to_add: Vec<(UncheckedK, UncheckedV)> = vec![entry.clone()];
        let to_remove: Vec<UncheckedK> = vec![entry.0];
        let msg = self.msg_builder(to_add, to_remove);

        let expected: Vec<(UncheckedK, UncheckedV)> = vec![];

        self.execute(deps.as_mut(), msg)?;

        self.assert_expected_entries(&deps.storage, expected);

        Ok(())
    }

    pub fn test_remove_nonexistent(&mut self, deps: &mut MockDeps) -> Result<(), TError> {
        let entry = self.mock_entry_builder();

        let to_add: Vec<(UncheckedK, UncheckedV)> = vec![];
        let to_remove: Vec<UncheckedK> = vec![entry.0];
        let msg = self.msg_builder(to_add, to_remove);

        let expected: Vec<(UncheckedK, UncheckedV)> = vec![];

        self.execute(deps.as_mut(), msg)?;

        self.assert_expected_entries(&deps.storage, expected);

        Ok(())
    }

    /// Run through all the preconfigured test scenarios
    pub fn test_all(&mut self, deps: &mut MockDeps) -> Result<(), TError> {
        self.test_add_one(deps)?;
        self.test_add_one_twice(deps)?;
        self.test_add_two_same(deps)?;
        self.test_add_and_remove_same(deps)?;
        self.test_remove_nonexistent(deps)?;

        Ok(())
    }

    /// Test the manually provided arguments with the expected behavior, which is removing any duplicate entries that are within both add and remove
    pub fn test_update_auto_expect(
        &mut self,
        deps: &mut MockDeps,
        update: (Vec<(UncheckedK, UncheckedV)>, Vec<UncheckedK>),
    ) -> Result<(), TError> {
        let (to_add, to_remove) = update;
        let msg = self.msg_builder(to_add.clone(), to_remove.clone());

        let expected: Vec<(UncheckedK, UncheckedV)> = determine_expected(&to_add, &to_remove);

        self.execute(deps.as_mut(), msg)?;

        self.assert_expected_entries(&deps.storage, expected);

        Ok(())
    }

    /// Provide an update nad expected result, and test that the expected result is returned
    pub fn test_update_with_expected(
        &mut self,
        deps: &mut MockDeps,
        update: (Vec<(UncheckedK, UncheckedV)>, Vec<UncheckedK>),
        expected: Vec<(UncheckedK, UncheckedV)>,
    ) -> Result<(), TError> {
        let (to_add, to_remove) = update;
        let msg = self.msg_builder(to_add, to_remove);

        self.execute(deps.as_mut(), msg)?;

        self.assert_expected_entries(&deps.storage, expected);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod determine_expected {
        use super::*;

        #[test]
        fn removes_in_to_add() {
            let to_add = vec![("a".to_string(), 1), ("b".to_string(), 2)];
            let to_remove = vec!["a".to_string()];

            let expected = vec![("b".to_string(), 2)];

            assert_that!(determine_expected(&to_add, &to_remove)).is_equal_to(expected);
        }

        #[test]
        fn removes_all() {
            let to_add = vec![("a".to_string(), 1), ("b".to_string(), 2)];
            let to_remove = vec!["a".to_string(), "b".to_string()];

            let expected: Vec<(String, i32)> = vec![];

            assert_that!(determine_expected(&to_add, &to_remove)).is_equal_to(expected);
        }

        #[test]
        fn empty() {
            let to_add: Vec<(String, i32)> = vec![];
            let to_remove: Vec<String> = vec![];

            let expected: Vec<(String, i32)> = vec![];

            assert_that!(determine_expected(&to_add, &to_remove)).is_equal_to(expected);
        }
    }

    mod sort_expected {}
}
