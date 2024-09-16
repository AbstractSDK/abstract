pub mod state {
    use cw_storage_plus::Item;

    use crate::objects::{ans_host::AnsHost, version_control::VersionControlContract};

    pub mod namespace {
        pub const CONFIG: &str = "a";
    }

    pub const CONFIG: Item<Config> = Item::new(namespace::CONFIG);

    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        pub version_control: VersionControlContract,
        pub ans_host: AnsHost,
    }
}
