use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for, export_schema_with_title};

use abstract_add_on::state::AddOnState;
use abstract_os::modules::add_ons::subscription::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse, ConfigResponse, ContributorStateResponse, SubscriberStateResponse, SubscriptionFeeResponse};
use cosmwasm_std::Binary;
use cw_asset::AssetInfo;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();


    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(AddOnState), &out_dir);
    export_schema(&schema_for!(StateResponse), &out_dir);
    export_schema(&schema_for!(ContributorStateResponse), &out_dir);
    export_schema(&schema_for!(SubscriberStateResponse), &out_dir);
    export_schema(&schema_for!(ConfigResponse), &out_dir);

    export_schema_with_title(
        &schema_for!(AssetInfo),
        &out_dir,
        "AssetInfoBase_for_Addr",
    );

    export_schema_with_title(
        &schema_for!(SubscriptionFeeResponse),
         &out_dir,
        "FeeResponse"
    );
}
