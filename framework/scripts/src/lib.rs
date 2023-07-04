pub mod assets;
pub mod contracts;
pub mod hashmap_diff;
pub mod pools;

use abstract_core::objects::UniquePoolId;
use cw_asset::AssetInfoBase;
use std::collections::HashMap;
use std::collections::HashSet;

use abstract_core::objects::UncheckedContractEntry;
use cw_orch::prelude::ContractInstance;
use cw_orch::state::ChainState;

use abstract_core::objects::pool_id::UncheckedPoolAddress;
use abstract_core::objects::PoolMetadata;
use abstract_interface::AnsHost;
use cw_orch::daemon::Daemon;

use abstract_interface::AbstractInterfaceError;

use reqwest::Client;
use serde_json::Value;
use tokio::runtime::Runtime;

const ANS_SCRAPE_URL: &str =
    "https://raw.githubusercontent.com/AbstractSDK/ans-scraper/mainline/out/";

/// get some json  
pub fn get_scraped_json_data(suffix: &str) -> Value {
    let client = Client::new();
    let url = format!("{}{}.json", ANS_SCRAPE_URL, suffix);
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let response = client.get(&url).send().await.unwrap();
        let json: Value = response.json().await.unwrap();
        json
    })
}

#[derive(Default)]
pub struct AnsData {
    pub contracts: HashMap<UncheckedContractEntry, String>,
    pub assets: HashMap<String, AssetInfoBase<String>>,
    // pub channels: Vec<(ChannelEntry, String)>,
    pub dexes: HashMap<String, String>, // We use this structure to work more easily with hash_map_diff::diff
    pub pools: HashMap<UncheckedPoolAddress, (UniquePoolId, PoolMetadata)>,
}

pub type EntryDif<K, V> = (HashSet<K>, HashMap<K, V>);

#[derive(Default)]
pub struct AnsDataDiff {
    pub contracts: EntryDif<UncheckedContractEntry, String>,
    pub assets: EntryDif<String, AssetInfoBase<String>>,
    // pub channels: Vec<(ChannelEntry, String)>,
    pub dexes: EntryDif<String, String>,
    pub pools: (
        HashSet<UniquePoolId>,
        HashMap<UncheckedPoolAddress, PoolMetadata>,
    ),
}

pub fn get_scraped_entries(ans_host: &AnsHost<Daemon>) -> Result<AnsData, AbstractInterfaceError> {
    let chain_name = &ans_host.get_chain().state().chain_data.chain_name;
    let chain_id = ans_host.get_chain().state().chain_data.chain_id.to_string();

    let contracts = crate::contracts::get_scraped_entries(chain_name, &chain_id)?;
    let assets = crate::assets::get_scraped_entries(chain_name, &chain_id)?;
    let (pools, dexes) = crate::pools::get_scraped_entries(chain_name, &chain_id)?;

    Ok(AnsData {
        contracts: contracts.into_iter().collect(),
        assets,
        dexes: dexes.into_iter().map(|v| (v.clone(), v)).collect(),
        pools: pools
            .into_iter()
            .map(|(a, m)| (a, (UniquePoolId::new(0), m)))
            .collect(),
    })
}

pub fn get_on_chain_entries(ans_host: &AnsHost<Daemon>) -> Result<AnsData, AbstractInterfaceError> {
    let contracts = crate::contracts::get_on_chain_entries(ans_host)?;
    let assets = crate::assets::get_on_chain_entries(ans_host)?;
    let pools = crate::pools::get_on_chain_entries(ans_host)?;
    let dexes = crate::pools::get_on_chain_dexes(ans_host)?;

    Ok(AnsData {
        contracts,
        assets,
        dexes: dexes.into_iter().map(|v| (v.clone(), v)).collect(),
        // For pools, we create a dummy unique ID for on-chain entities
        pools,
    })
}

pub fn diff(
    scraped_entry: AnsData,
    on_chain_entry: AnsData,
) -> Result<AnsDataDiff, AbstractInterfaceError> {
    let contracts = crate::hashmap_diff::diff(scraped_entry.contracts, on_chain_entry.contracts)?;
    let assets = crate::hashmap_diff::diff(scraped_entry.assets, on_chain_entry.assets)?;
    let dexes =
        crate::hashmap_diff::diff(scraped_entry.dexes.clone(), on_chain_entry.dexes.clone())?;

    // For pools, we diff only the metadata and then get the uniquepoolid to attach to the address
    let pools = crate::hashmap_diff::diff(
        scraped_entry
            .pools
            .iter()
            .map(|(a, (_u, m))| (a.clone(), m.clone()))
            .collect(),
        on_chain_entry
            .pools
            .iter()
            .map(|(a, (_u, m))| (a.clone(), m.clone()))
            .collect(),
    )?;

    let pool_return = (
        pools
            .0
            .iter()
            .map(|k| on_chain_entry.pools.get(k).unwrap().0)
            .collect(),
        pools.1,
    );

    Ok(AnsDataDiff {
        contracts,
        assets,
        pools: pool_return,
        dexes,
    })
}

pub fn update(ans_host: &AnsHost<Daemon>, diff: AnsDataDiff) -> Result<(), AbstractInterfaceError> {
    contracts::update(ans_host, diff.contracts)?;
    assets::update(ans_host, diff.assets)?;
    pools::update_dexes(ans_host, diff.dexes)?;
    pools::update(ans_host, diff.pools)?;

    Ok(())
}
