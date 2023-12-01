use crate::EntryDif;
use cw_orch::prelude::*;

use abstract_core::ans_host::*;
use abstract_core::objects::pool_id::{PoolAddressBase, UncheckedPoolAddress};
use abstract_core::objects::{DexAssetPairing, PoolMetadata, UniquePoolId};
use abstract_interface::{AbstractInterfaceError, AnsHost};

use serde_json::Value;
use std::collections::{HashMap, HashSet};

pub type ScrapedEntries = (
    HashMap<PoolAddressBase<String>, PoolMetadata>,
    HashSet<String>,
);

pub fn get_scraped_entries(
    chain_name: &String,
    chain_id: &String,
) -> Result<ScrapedEntries, AbstractInterfaceError> {
    let raw_scraped_entries = crate::get_scraped_json_data("pools");

    let binding = raw_scraped_entries[chain_name][chain_id].clone();
    let parsed_scraped_entries: &Vec<Value> = binding.as_array().unwrap();
    let mut dexes_to_register: HashSet<String> = HashSet::new();

    let scraped_entries_vec: Vec<(UncheckedPoolAddress, PoolMetadata)> = parsed_scraped_entries
        .iter()
        .map(|value| {
            let pool: (UncheckedPoolAddress, PoolMetadata) =
                serde_json::from_value(value.clone()).unwrap();

            dexes_to_register.insert(pool.1.dex.clone());

            pool
        })
        .collect();

    Ok((scraped_entries_vec.into_iter().collect(), dexes_to_register))
}

pub fn get_on_chain_entries(
    ans_host: &AnsHost<Daemon>,
) -> Result<HashMap<PoolAddressBase<String>, (UniquePoolId, PoolMetadata)>, AbstractInterfaceError>
{
    let mut on_chain_entries = HashMap::new();
    let mut last_pool = None;
    loop {
        let PoolMetadataListResponse { metadatas } =
            ans_host.pool_metadata_list(None, Some(100), last_pool)?;
        if metadatas.is_empty() {
            break;
        }

        let addresses: Vec<_> = ans_host
            .pools(
                metadatas
                    .iter()
                    .map(|(_, m)| {
                        DexAssetPairing::new(m.assets[0].clone(), m.assets[1].clone(), &m.dex)
                    })
                    .collect(),
            )?
            .pools;

        let metadata_to_save: HashMap<_, _> = metadatas
            .iter()
            .zip(addresses.iter())
            .flat_map(|(m, a)| {
                a.1.iter()
                    .map(|a| (a.pool_address.clone().into(), m.clone()))
            })
            .collect();

        last_pool = metadatas.last().map(|l| l.0);
        on_chain_entries.extend(metadata_to_save);
    }

    Ok(on_chain_entries)
}

pub fn get_on_chain_dexes(
    ans_host: &AnsHost<Daemon>,
) -> Result<Vec<String>, AbstractInterfaceError> {
    let RegisteredDexesResponse { dexes } = ans_host.registered_dexes()?;
    Ok(dexes)
}

pub fn update(
    ans_host: &AnsHost<Daemon>,
    diff: (
        HashSet<UniquePoolId>,
        HashMap<UncheckedPoolAddress, PoolMetadata>,
    ),
) -> Result<(), AbstractInterfaceError> {
    let to_add: Vec<_> = diff.1.into_iter().collect();
    let to_remove: Vec<_> = diff.0.into_iter().collect();

    // add the pools
    ans_host.execute_chunked(&to_add.into_iter().collect::<Vec<_>>(), 25, |chunk| {
        ExecuteMsg::UpdatePools {
            to_add: chunk.to_vec(),
            to_remove: vec![],
        }
    })?;

    // remove the pools
    ans_host.execute_chunked(&to_remove.into_iter().collect::<Vec<_>>(), 25, |chunk| {
        ExecuteMsg::UpdatePools {
            to_add: vec![],
            to_remove: chunk.to_vec(),
        }
    })?;

    Ok(())
}

pub fn update_dexes(
    ans_host: &AnsHost<Daemon>,
    diff: EntryDif<String, String>,
) -> Result<(), AbstractInterfaceError> {
    let to_add: Vec<_> = diff.1.into_keys().collect();
    let to_remove: Vec<_> = diff.0.into_iter().collect();

    // add the dexes
    ans_host.execute_chunked(&to_add, 25, |chunk| ExecuteMsg::UpdateDexes {
        to_add: chunk.to_vec(),
        to_remove: vec![],
    })?;

    // remove the dexes
    ans_host.execute_chunked(&to_remove, 25, |chunk| ExecuteMsg::UpdateDexes {
        to_add: vec![],
        to_remove: chunk.to_vec(),
    })?;

    Ok(())
}
