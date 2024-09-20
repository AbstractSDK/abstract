pub mod state {
    use cw_storage_plus::Item;

    use crate::objects::{
        ans_host::AnsHost, storage_namespaces, version_control::VersionControlContract,
    };

    pub const CONFIG: Item<Config> = Item::new(storage_namespaces::CONFIG_STORAGE_KEY);

    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        pub version_control: VersionControlContract,
        pub ans_host: AnsHost,
    }
}
