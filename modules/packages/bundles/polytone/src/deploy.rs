// Start by uploading the voice, note and proxy contract

// Then instantiate the voice and note

use crate::interface::{Note, PolytoneAccount, Proxy, Voice};
use anyhow::Result as AnyResult;
use bech32::{FromBase32, ToBase32, Variant};
use cosmwasm_std::{instantiate2_address, Addr, Binary};
use cw_orch::{
    interchain::{interchain_channel::InterchainChannel, interchain_env::contract_port},
    prelude::{
        interchain_channel_builder::InterchainChannelBuilder, queriers::CosmWasm, ContractInstance,
        CwOrchInstantiate, CwOrchUpload, Daemon, InterchainEnv,
    },
    starship::Starship,
    state::ChainState,
    tokio::runtime::Runtime,
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
) -> AnyResult<PolytoneAccount<Daemon>> {
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
    // Once the channel is created, we need to get the proxy address

    let proxy_addr = rt.block_on(proxy_address(&interchain_channel, &note, &proxy))?;

    proxy.set_address(&Addr::unchecked(proxy_addr));

    Ok(PolytoneAccount { note, voice })
}

pub async fn proxy_address(
    interchain_channel: &InterchainChannel,
    note: &Note<Daemon>,
    proxy: &Proxy<Daemon>,
) -> AnyResult<String> {
    // We start with the proxy_code_id_checksum
    let code_info = proxy
        .get_chain()
        .query_client::<CosmWasm>()
        .code(proxy.code_id()?)
        .await?;

    let checksum = code_info.data_hash;

    let canon_contract_addr = bech32::decode(note.addr_str()?.as_str())?.1;

    let salt = salt(
        &interchain_channel.get_connection(),
        contract_port(note).to_string().as_str(),
        note.get_chain().daemon.sender().as_str(),
    );
    let proxy_canon_addr = instantiate2_address(
        &checksum,
        &Vec::<u8>::from_base32(&canon_contract_addr)?.into(),
        &salt,
    )?;

    let proxy_human_addr = bech32::encode(
        proxy
            .get_chain()
            .state()
            .0
            .chain_data
            .bech32_prefix
            .as_str(),
        proxy_canon_addr.0 .0.to_base32(),
        Variant::Bech32,
    )?;

    Ok(proxy_human_addr)
}

// Copied from polytone implementation
fn salt(local_connection: &str, counterparty_port: &str, remote_sender: &str) -> Binary {
    use sha2::{Digest, Sha512};
    // the salt can be a max of 64 bytes (512 bits).
    let hash = Sha512::default()
        .chain_update(local_connection.as_bytes())
        .chain_update(counterparty_port.as_bytes())
        .chain_update(remote_sender.as_bytes())
        .finalize();
    Binary::from(hash.as_slice())
}

#[test]
fn polytone_deploy() -> AnyResult<()> {
    env_logger::init();
    let rt = Runtime::new()?;
    let starship = Starship::new(rt.handle().to_owned(), None)?;
    deploy(&rt, &starship, "juno-1", "osmosis-1", "1".to_string())?;
    Ok(())
}
