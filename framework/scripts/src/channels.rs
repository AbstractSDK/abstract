use abstract_core::ans_host::*;
use abstract_core::objects::UncheckedChannelEntry;
use abstract_interface::{AbstractInterfaceError, AnsHost};
use serde_json::from_reader;
use std::fs::File;

use cw_orch::prelude::*;
use cw_orch::state::ChainState;

const PATH: &str = "resources/old/channels.json";

pub fn update_channels(ans: &AnsHost<Daemon>) -> Result<(), AbstractInterfaceError> {
    let file = File::open(PATH).unwrap_or_else(|_| panic!("file should be present at {}", PATH));
    let json: serde_json::Value = from_reader(file)?;
    let chain_name = &ans.get_chain().state().chain_data.chain_name;
    let chain_id = ans.get_chain().state().chain_data.chain_id.to_string();
    let channels = json
        .get(chain_name)
        .unwrap()
        .get(chain_id)
        .ok_or_else(|| CwOrchError::StdErr("network not found".into()))?;

    let channels = channels.as_object().unwrap();
    let channels_to_add: Vec<(UncheckedChannelEntry, String)> = channels
        .iter()
        .map(|(name, value)| {
            let id = value.as_str().unwrap().to_owned();
            let key = UncheckedChannelEntry::try_from(name.clone()).unwrap();
            (key, id)
        })
        .collect();

    ans.execute_chunked(&channels_to_add, 25, |chunk| ExecuteMsg::UpdateChannels {
        to_add: chunk.to_vec(),
        to_remove: vec![],
    })?;

    Ok(())
}
