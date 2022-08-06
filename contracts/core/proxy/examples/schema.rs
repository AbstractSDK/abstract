use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};

use abstract_os::objects::proxy_asset::{ProxyAsset, UncheckedProxyAsset};
use abstract_os::proxy::state::State;
use abstract_os::proxy::{
    ExecuteMsg, InstantiateMsg, QueryConfigResponse, QueryHoldingAmountResponse,
    QueryHoldingValueResponse, QueryMsg, QueryProxyAssetConfigResponse, QueryProxyAssetsResponse,
    QueryTotalValueResponse, QueryValidityResponse,
};
use cosmwasm_std::{CosmosMsg, Empty};
use cw_asset::AssetInfo;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(State), &out_dir);
    export_schema(&schema_for!(ProxyAsset), &out_dir);
    export_schema(&schema_for!(UncheckedProxyAsset), &out_dir);
    export_schema_with_title(&schema_for!(QueryConfigResponse), &out_dir, "ConfigResponse");
    export_schema_with_title(&schema_for!(QueryTotalValueResponse), &out_dir, "TotalValueResponse");
    export_schema_with_title(&schema_for!(QueryHoldingValueResponse), &out_dir, "HoldingValueResponse");
    export_schema_with_title(&schema_for!(QueryHoldingAmountResponse), &out_dir, "HoldingAmountResponse");
    export_schema_with_title(&schema_for!(QueryProxyAssetConfigResponse), &out_dir, "ProxyAssetConfigResponse");
    export_schema_with_title(&schema_for!(QueryProxyAssetsResponse), &out_dir, "ProxyAssetsResponse");
    export_schema_with_title(&schema_for!(QueryValidityResponse), &out_dir, "CheckValidityResponse");

    export_schema_with_title(
        &schema_for!(CosmosMsg<Empty>),
        &out_dir,
        "CosmosMsg_for_Empty",
    );

    export_schema_with_title(&schema_for!(AssetInfo), &out_dir, "AssetInfoBase_for_Addr");
}
