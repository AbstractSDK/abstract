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

/// State of abstract deployments
pub struct AbstractDaemonState(serde_json::Value);

impl Default for AbstractDaemonState {
    fn default() -> Self {
        Self(State::load_state())
    }
}

impl AbstractDaemonState {
    /// Get address of the abstract contract by contract_id
    pub fn contract_addr(&self, chain_id: &str, contract_id: &str) -> Option<cosmwasm_std::Addr> {
        self.0[chain_id]["default"][contract_id]
            .as_str()
            .map(cosmwasm_std::Addr::unchecked)
    }

    /// Get code id of the abstract contract by contract_id
    pub fn contract_code_id(&self, chain_id: &str, contract_id: &str) -> Option<u64> {
        self.0[chain_id]["code_ids"][contract_id].as_u64()
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
        let vc_juno = state.contract_code_id("pion-1", REGISTRY);
        assert!(vc_juno.is_some());
    }
}
