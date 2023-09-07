use crate::EntryDif;
use cw_asset::AssetInfoBase;

use cw_orch::prelude::*;

use abstract_core::ans_host::*;
use abstract_interface::{AbstractInterfaceError, AnsHost};

use serde_json::{from_value, Value};
use std::collections::HashMap;

pub fn get_scraped_entries(
    chain_name: &str,
    chain_id: &str,
) -> Result<HashMap<String, AssetInfoBase<String>>, AbstractInterfaceError> {
    let raw_scraped_entries = crate::get_scraped_json_data("assets");

    let parsed_scraped_entries: Vec<Vec<Value>> =
        from_value(raw_scraped_entries[chain_name][chain_id].clone()).unwrap();

    let scraped_entries_vec: Vec<(String, AssetInfoBase<String>)> = parsed_scraped_entries
        .into_iter()
        .map(|v| {
            let asset_info: AssetInfoBase<String> = from_value(v[1].clone()).unwrap();
            (v[0].as_str().unwrap().to_owned(), asset_info)
        })
        .collect();

    Ok(scraped_entries_vec.into_iter().collect())
}

pub fn get_on_chain_entries(
    ans_host: &AnsHost<Daemon>,
) -> Result<HashMap<String, AssetInfoBase<String>>, AbstractInterfaceError> {
    let mut on_chain_entries = HashMap::new();
    let mut last_asset = None;
    loop {
        let AssetListResponse { assets } = ans_host.asset_list(None, None, last_asset)?;
        if assets.is_empty() {
            break;
        }
        last_asset = assets.last().map(|(entry, _)| entry.to_string());
        on_chain_entries.extend(assets.into_iter().map(|(a, b)| (a.to_string(), b.into())));
    }

    Ok(on_chain_entries)
}

pub fn update(
    ans_host: &AnsHost<Daemon>,
    diff: EntryDif<String, AssetInfoBase<String>>,
) -> Result<(), AbstractInterfaceError> {
    let to_add: Vec<_> = diff.1.into_iter().collect();
    let to_remove: Vec<_> = diff.0.into_iter().collect();

    // add the assets
    ans_host.execute_chunked(&to_add, 25, |chunk| ExecuteMsg::UpdateAssetAddresses {
        to_add: chunk.to_vec(),
        to_remove: vec![],
    })?;

    // remove the assets
    ans_host.execute_chunked(&to_remove, 25, |chunk| ExecuteMsg::UpdateAssetAddresses {
        to_add: vec![],
        to_remove: chunk.to_vec(),
    })?;

    Ok(())
}

#[cfg(test)]
mod test {

    use cw_orch::daemon::ChainRegistryData as ChainData;
    use std::env::set_var;
    use tokio::runtime::Runtime;

    use super::{get_on_chain_entries, get_scraped_entries};
    use abstract_interface::Abstract;
    use anyhow::Result as AnyResult;
    use cw_orch::daemon::{ChainInfo, DaemonBuilder};
    use cw_orch::{deploy::Deploy, prelude::networks::JUNO_1};
    const CHAIN: ChainInfo = JUNO_1;

    #[test]
    fn scraped_data_exists() {
        let chain: ChainData = CHAIN.into();

        let chain_name = chain.chain_name;
        let chain_id = chain.chain_id.to_string();

        let scraped = get_scraped_entries(&chain_name, &chain_id).unwrap();

        println!("scraped: {scraped:?}");

        // TODO, we could add better tests
        assert!(!scraped.is_empty());
        // We would have juno asset for sure
        assert_eq!(
            scraped.get("juno>juno").unwrap().to_owned(),
            cw_asset::AssetInfoBase::Native(CHAIN.gas_denom.to_owned())
        );
    }

    #[test]
    fn on_chain_data_exists() -> AnyResult<()> {
        // We setup a dummy main mnemonic for the daemon
        set_var("MAIN_MNEMONIC", "proof truly city acoustic walnut thrive seat illegal recycle quote kite pudding clarify limit black evidence dove lens velvet penalty glance ghost fog ship");

        let rt = Runtime::new()?;

        let chain = DaemonBuilder::default()
            .handle(rt.handle())
            .chain(CHAIN)
            .build()?;

        let deployment = Abstract::load_from(chain)?;
        // Take the assets, contracts, and pools from resources and upload them to the ans host
        let ans_host = deployment.ans_host;
        let on_chain = get_on_chain_entries(&ans_host).unwrap();
        // TODO, we could add better tests
        assert!(!on_chain.is_empty());
        assert_eq!(
            on_chain.get("juno>juno").unwrap().to_owned(),
            cw_asset::AssetInfoBase::Native(CHAIN.gas_denom.to_owned())
        );
        Ok(())
    }
}
