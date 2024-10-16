use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "./"]
#[include = "state.json"]
struct State;

impl State {
    pub fn load_state() -> serde_json::Value {
        let state_file =
            State::get("state.json").expect("Unable to read abstract-interface state.json");
        serde_json::from_slice(&state_file.data).unwrap()
    }
}

use cw_orch::environment::EnvironmentInfo;

/// State of abstract deployments
pub struct AbstractDaemonState(serde_json::Value);

impl Default for AbstractDaemonState {
    fn default() -> Self {
        Self(State::load_state())
    }
}

impl AbstractDaemonState {
    /// Get address of the abstract contract by contract_id
    pub fn contract_addr(
        &self,
        env_info: &EnvironmentInfo,
        contract_id: &str,
    ) -> Option<cosmwasm_std::Addr> {
        self.0[&env_info.chain_id][&env_info.deployment_id][contract_id]
            .as_str()
            .map(cosmwasm_std::Addr::unchecked)
    }

    /// Get code id of the abstract contract by contract_id
    pub fn contract_code_id(&self, env_info: &EnvironmentInfo, contract_id: &str) -> Option<u64> {
        self.0[&env_info.chain_id]["code_ids"][contract_id].as_u64()
    }

    /// Get raw state of the abstract deployments
    pub fn state(&self) -> serde_json::Value {
        self.0.clone()
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use std::borrow::Cow;

    use abstract_std::REGISTRY;

    use super::*;

    #[test]
    fn only_state_json_included() {
        let files = State::iter().collect::<Vec<_>>();
        assert_eq!(files, vec![Cow::Borrowed("state.json")]);
        State::get("state.json").unwrap();
    }

    #[test]
    fn have_some_state() {
        let state = AbstractDaemonState::default();
        let vc_juno = state.contract_code_id(
            &EnvironmentInfo {
                chain_id: "pion-1".to_owned(),
                chain_name: "neutron-testnet".to_owned(),
                deployment_id: "default".to_owned(),
            },
            REGISTRY,
        );
        assert!(vc_juno.is_some());
    }
}
