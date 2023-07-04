// use cw_orch::{
//     prelude::{
//         *,
//     },
// };
// use tokio::runtime::Runtime;

// pub const ABSTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// const ANS_SCRAPE_URL: &str =
//     "https://raw.githubusercontent.com/AbstractSDK/ans-scraper/mainline/out/";

// pub mod ans_merge {
//     use super::*;
//     use abstract_core::ans_host::*;
//     use abstract_interface::{AbstractInterfaceError, AnsHost};
//     use cw_asset::{AssetInfo, AssetInfoUnchecked};
//     use cw_orch::state::ChainState;
//     use reqwest::Client;
//     use serde_json::{from_value, Value};
//     use std::collections::{HashMap, HashSet};
//     use std::str::FromStr;

//     fn get_scraped_entries(
//         chain_name: &String,
//         chain_id: &String,
//     ) -> Result<HashMap<String, String>, AbstractInterfaceError> {
//         let raw_scraped_entries = get_scraped_json_data("assets");
//         println!(
//             "scraped_entries: {:?}",
//             raw_scraped_entries[chain_name][chain_id]
//         );

//         let parsed_scraped_entries: Vec<Vec<Value>> =
//             from_value(raw_scraped_entries[chain_name][chain_id].clone()).unwrap();

//         let scraped_entries_vec: Vec<(String, String)> = parsed_scraped_entries
//             .into_iter()
//             .map(|v| {
//                 let asset_info: AssetInfo = from_value(v[1].clone()).unwrap();
//                 (v[0].as_str().unwrap().to_owned(), asset_info.to_string())
//             })
//             .collect();

//         Ok(scraped_entries_vec.into_iter().collect())
//     }

//     fn get_on_chain_entries(
//         ans_host: &AnsHost<Daemon>,
//     ) -> Result<HashMap<String, String>, AbstractInterfaceError> {
//         let mut on_chain_entries = HashMap::new();
//         let mut last_asset = None;
//         loop {
//             let AssetListResponse { assets } = ans_host.asset_list(None, None, last_asset)?;
//             if assets.is_empty() {
//                 break;
//             }
//             last_asset = assets.last().map(|(entry, _)| entry.to_string());
//             on_chain_entries.extend(
//                 assets
//                     .into_iter()
//                     .map(|(a, b)| (a.to_string(), b.to_string())),
//             );
//         }

//         Ok(on_chain_entries)
//     }

//     fn get_union_keys<'a>(
//         scraped_entries: &'a HashMap<String, String>,
//         on_chain_entries: &'a HashMap<String, String>,
//     ) -> Vec<&'a String> {
//         let on_chain_binding = on_chain_entries.keys().collect::<HashSet<_>>();
//         let scraped_binding = scraped_entries.keys().collect::<HashSet<_>>();

//         on_chain_binding.union(&scraped_binding).cloned().collect()
//     }

//     fn get_assets_changes(
//         union_keys: &Vec<&String>,
//         scraped_entries: &HashMap<String, String>,
//         on_chain_entries: &HashMap<String, String>,
//     ) -> (Vec<String>, Vec<(String, cw_asset::AssetInfoBase<String>)>) {
//         let mut assets_to_remove: Vec<String> = vec![];
//         let mut assets_to_add: Vec<(String, cw_asset::AssetInfoBase<String>)> = vec![];

//         for entry in union_keys {
//             if !scraped_entries.contains_key(entry.as_str()) {
//                 assets_to_remove.push((*entry).to_string())
//             }

//             if !on_chain_entries.contains_key(*entry) {
//                 if let Ok(info) = AssetInfoUnchecked::from_str(scraped_entries.get(*entry).unwrap())
//                 {
//                     assets_to_add.push(((*entry).to_owned(), info))
//                 }
//             }
//         }
//         return (assets_to_remove, assets_to_add);
//     }

//     fn update_channels(ans: &AnsHost<Daemon>) -> Result<(), crate::CwOrchError> {
//         let file =
//             File::open(&path).unwrap_or_else(|_| panic!("file should be present at {}", &path));
//         let json: serde_json::Value = from_reader(file)?;
//         let chain_name = &ans.get_chain().state().chain_data.chain_name;
//         let chain_id = ans.get_chain().state().chain_data.chain_id.to_string();
//         let channels = json
//             .get(chain_name)
//             .unwrap()
//             .get(chain_id)
//             .ok_or_else(|| CwOrchError::StdErr("network not found".into()))?;

//         let channels = channels.as_object().unwrap();
//         let channels_to_add: Vec<(UncheckedChannelEntry, String)> = channels
//             .iter()
//             .map(|(name, value)| {
//                 let id = value.as_str().unwrap().to_owned();
//                 let key = UncheckedChannelEntry::try_from(name.clone()).unwrap();
//                 (key, id)
//             })
//             .collect();

//         ans.execute_chunked(&channels_to_add, 25, |chunk| ExecuteMsg::UpdateChannels {
//             to_add: chunk.to_vec(),
//             to_remove: vec![],
//         })?;

//         Ok(())
//     }

// }