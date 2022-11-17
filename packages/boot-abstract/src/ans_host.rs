use std::{cmp::min, env, fs::File};

use abstract_sdk::os::{
    ans_host::*,
    objects::{UncheckedChannelEntry, UncheckedContractEntry},
};

use cw_asset::AssetInfoUnchecked;

use serde_json::from_reader;

use crate::AbstractOS;
use boot_core::{BootError, Contract, Daemon, IndexResponse, TxHandler, TxResponse};

pub type AnsHost<Chain> = AbstractOS<Chain, ExecuteMsg, InstantiateMsg, QueryMsg, MigrateMsg>;

impl<Chain: TxHandler + Clone> AnsHost<Chain>
where
    TxResponse<Chain>: IndexResponse,
{
    pub fn new(name: &str, chain: &Chain) -> Self {
        Self(
            Contract::new(name, chain).with_wasm_path("ans_host"), // .with_mock(Box::new(
                                                                   //     ContractWrapper::new_with_empty(
                                                                   //         ::contract::execute,
                                                                   //         ::contract::instantiate,
                                                                   //         ::contract::query,
                                                                   //     ),
                                                                   // ))
        )
    }
}

impl AnsHost<Daemon> {
    pub fn update_all(&self) -> Result<(), BootError> {
        self.update_assets()?;
        self.update_contracts()?;
        Ok(())
    }

    pub fn update_assets(&self) -> Result<(), BootError> {
        let path = env::var("ANS_HOST_ASSETS")?;
        let file =
            File::open(&path).unwrap_or_else(|_| panic!("file should be present at {}", &path));
        let json: serde_json::Value = from_reader(file)?;
        let chain_id = self.0.chain().state.chain.chain_id;
        let network_id = self.0.chain().state.id.clone();
        let maybe_assets = json.get(chain_id).unwrap().get(network_id);

        match maybe_assets {
            Some(assets_value) => {
                let assets = assets_value.as_object().unwrap();
                let to_add: Vec<(String, AssetInfoUnchecked)> = assets
                    .iter()
                    .map(|(name, value)| {
                        let asset: AssetInfoUnchecked =
                            serde_json::from_value(value.clone()).unwrap();
                        (name.clone(), asset)
                    })
                    .collect();
                let mut i = 0;
                while i != to_add.len() {
                    let chunk = to_add.get(i..min(i + 25, to_add.len())).unwrap();
                    i += chunk.len();
                    self.execute(
                        &ExecuteMsg::UpdateAssetAddresses {
                            to_add: chunk.to_vec(),
                            to_remove: vec![],
                        },
                        None,
                    )?;
                }

                Ok(())
            }
            None => Err(BootError::StdErr("network not found".into())),
        }
    }

    pub fn update_channels(&self) -> Result<(), BootError> {
        let path = env::var("ANS_HOST_CHANNELS")?;
        let file =
            File::open(&path).unwrap_or_else(|_| panic!("file should be present at {}", &path));
        let json: serde_json::Value = from_reader(file)?;
        let chain_id = self.0.chain().state.chain.chain_id;
        let network_id = self.0.chain().state.id.clone();
        let maybe_channels = json.get(chain_id).unwrap().get(network_id);

        match maybe_channels {
            Some(channels_value) => {
                let channels = channels_value.as_object().unwrap();
                let to_add: Vec<(UncheckedChannelEntry, String)> = channels
                    .iter()
                    .map(|(name, value)| {
                        let id = value.as_str().unwrap().to_owned();
                        let key = UncheckedChannelEntry::try_from(name.clone()).unwrap();
                        (key, id)
                    })
                    .collect();
                let mut i = 0;
                while i < to_add.len() {
                    let chunk = to_add.get(i..min(i + 25, to_add.len())).unwrap();
                    i += chunk.len();
                    self.execute(
                        &ExecuteMsg::UpdateChannels {
                            to_add: chunk.to_vec(),
                            to_remove: vec![],
                        },
                        None,
                    )?;
                }

                Ok(())
            }
            None => Err(BootError::StdErr("network not found".into())),
        }
    }

    pub fn update_contracts(&self) -> Result<(), BootError> {
        let path = env::var("ANS_HOST_CONTRACTS")?;

        let file =
            File::open(&path).unwrap_or_else(|_| panic!("file should be present at {}", &path));
        let json: serde_json::Value = from_reader(file)?;
        let chain_id = self.0.chain().state.chain.chain_id;
        let network_id = self.0.chain().state.id.clone();
        let maybe_contracts = json.get(chain_id).unwrap().get(network_id);

        match maybe_contracts {
            Some(contracts_value) => {
                let contracts = contracts_value.as_object().unwrap();
                let to_add: Vec<(UncheckedContractEntry, String)> = contracts
                    .iter()
                    .map(|(name, value)| {
                        let id = value.as_str().unwrap().to_owned();
                        let key = UncheckedContractEntry::try_from(name.clone()).unwrap();
                        (key, id)
                    })
                    .collect();
                let mut i = 0;
                while i < to_add.len() {
                    let chunk = to_add.get(i..min(i + 25, to_add.len())).unwrap();
                    i += chunk.len();
                    self.0.execute(
                        &ExecuteMsg::UpdateContractAddresses {
                            to_add: chunk.to_vec(),
                            to_remove: vec![],
                        },
                        None,
                    )?;
                }

                Ok(())
            }
            None => Err(BootError::StdErr("network not found".into())),
        }
    }
}
