// Start by uploading the voice, note and proxy contract

// Then instantiate the voice and note


use crate::interface::{Note, Polytone, Proxy, Voice};
use anyhow::Result as AnyResult;


use cw_orch::{
    prelude::{
        interchain_channel_builder::InterchainChannelBuilder, ContractInstance,
        CwOrchInstantiate, CwOrchUpload, Daemon, InterchainEnv, CwOrchExecute,
    },
    starship::Starship,
    tokio::runtime::Runtime, deploy::Deploy,
};

pub const MAX_BLOCK_GAS: u64 = 100_000_000;
pub const POLYTONE_VERSION: &str = "polytone-1";

// This is to be used with starship only (for testing)
pub fn deploy(
    rt: &Runtime,
    starship: &Starship,
    source_id: &str,
    dest_id: &str,
    id: String,
) -> AnyResult<Polytone<Daemon>> {
    let interchain: InterchainEnv = starship.interchain_env();

    let source = interchain.daemon(source_id)?;
    let dest = interchain.daemon(dest_id)?;

    let note = Note::new(format!("polytone:note-{}", id), source.clone());
    let voice = Voice::new(format!("polytone:voice-{}", id), dest.clone());
    let proxy = Proxy::new(format!("polytone:proxy-{}", id), dest.clone());

    note.upload()?;
    voice.upload()?;
    proxy.upload()?;

    note.instantiate(
        &polytone_note::msg::InstantiateMsg {
            pair: None,
            block_max_gas: MAX_BLOCK_GAS.into(),
        },
        None,
        None,
    )?;

    voice.instantiate(
        &polytone_voice::msg::InstantiateMsg {
            proxy_code_id: proxy.code_id()?.into(),
            block_max_gas: MAX_BLOCK_GAS.into(),
        },
        None,
        None,
    )?;

    // We need to create a channel between the two contracts
    let interchain_channel = rt.block_on(
        InterchainChannelBuilder::default()
            .from_contracts(&note, &voice)
            .create_channel(starship.client(), POLYTONE_VERSION),
    )?;   

    Ok(Polytone { note, voice, channel: interchain_channel })
}

#[test]
fn polytone_deploy() -> AnyResult<()> {
    env_logger::init();
    let rt = Runtime::new()?;
    let starship = Starship::new(rt.handle().to_owned(), None)?;
    let polytone = deploy(&rt, &starship, "juno-1", "osmosis-1", "1".to_string())?;

    // Now we test an interaction through the interchain

    let result = polytone.note.execute(&polytone_note::msg::ExecuteMsg::Execute { msgs: vec![], callback: None, timeout_seconds: 1_000_000u64.into() }, None)?;
    rt.block_on(polytone.channel.await_ibc_execution("juno-1".into(), result.txhash))?;

    Ok(())
}