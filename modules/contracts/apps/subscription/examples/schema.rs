use std::{env::current_dir, fs::create_dir_all};

use abstract_subscription::{contract::SubscriptionApp, msg::CustomExecuteMsg};
use cosmwasm_schema::remove_schemas;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    #[cfg(feature = "schema")]
    SubscriptionApp::export_schema_custom::<CustomExecuteMsg>(&out_dir);
}
