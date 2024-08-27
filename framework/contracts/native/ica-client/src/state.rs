use abstract_sdk::feature_objects::{AnsHost, VersionControlContract};
use cw_storage_plus::Item;

#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub version_control: VersionControlContract,
    pub ans_host: AnsHost,
}

pub const CONFIG: Item<Config> = Item::new("config");
