use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};

use abstract_os::subscription::{
    ConfigResponse, ContributorStateResponse, StateResponse, SubscriberStateResponse,
    SubscriptionFeeResponse,
};

use cw_asset::{AssetInfo, AssetInfoUnchecked};
use subscription::contract::SubscriptionAddOn;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    SubscriptionAddOn::export_schema(&out_dir);

    export_schema(&schema_for!(ConfigResponse), &out_dir);
    export_schema(&schema_for!(StateResponse), &out_dir);
    export_schema(&schema_for!(ContributorStateResponse), &out_dir);
    export_schema(&schema_for!(SubscriberStateResponse), &out_dir);

    export_schema_with_title(&schema_for!(AssetInfo), &out_dir, "AssetInfoBase_for_Addr");
    export_schema_with_title(
        &schema_for!(AssetInfoUnchecked),
        &out_dir,
        "AssetInfoBase_for_String",
    );

    export_schema_with_title(
        &schema_for!(SubscriptionFeeResponse),
        &out_dir,
        "FeeResponse",
    );
}
