use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};

use abstract_add_on::state::AddOnState;
use abstract_os::{
    add_on::{AddOnConfigResponse, BaseQueryMsg},
    subscription::{
        ConfigResponse, ContributorStateResponse, ExecuteMsg, InstantiateMsg, QueryMsg,
        StateResponse, SubscriberStateResponse, SubscriptionFeeResponse,
    },
};

use cw_asset::{AssetInfo, AssetInfoUnchecked};
use cw_controllers::AdminResponse;

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

    // Base add-on exports
    export_schema(&schema_for!(BaseQueryMsg), &out_dir);
    export_schema(&schema_for!(AddOnConfigResponse), &out_dir);
    export_schema(&schema_for!(AdminResponse), &out_dir);

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
