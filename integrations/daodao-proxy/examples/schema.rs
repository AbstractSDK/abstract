use abstract_sdk::core::proxy::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use cosmwasm_schema::write_api;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
    };
    // let mut out_dir = current_dir().unwrap();
    // out_dir.push("schema");
    // create_dir_all(&out_dir).unwrap();
    // remove_schemas(&out_dir).unwrap();

    // export_schema(&schema_for!(InstantiateMsg), &out_dir);
    // export_schema(&schema_for!(ExecuteMsg), &out_dir);
    // export_schema(&schema_for!(QueryMsg), &out_dir);
    // export_schema(&schema_for!(State), &out_dir);
    // export_schema(&schema_for!(ProxyAsset), &out_dir);
    // export_schema(&schema_for!(UncheckedProxyAsset), &out_dir);
    // export_schema(&schema_for!(BaseAssetResponse), &out_dir);
    // export_schema_with_title(&schema_for!(ConfigResponse), &out_dir, "ConfigResponse");
    // export_schema_with_title(
    //     &schema_for!(TotalValueResponse),
    //     &out_dir,
    //     "TotalValueResponse",
    // );
    // export_schema_with_title(
    //     &schema_for!(HoldingValueResponse),
    //     &out_dir,
    //     "HoldingValueResponse",
    // );
    // export_schema_with_title(
    //     &schema_for!(HoldingAmountResponse),
    //     &out_dir,
    //     "HoldingAmountResponse",
    // );
    // export_schema_with_title(
    //     &schema_for!(AssetConfigResponse),
    //     &out_dir,
    //     "AssetConfigResponse",
    // );
    // export_schema_with_title(&schema_for!(AssetsResponse), &out_dir, "AssetsResponse");
    // export_schema_with_title(
    //     &schema_for!(ValidityResponse),
    //     &out_dir,
    //     "CheckValidityResponse",
    // );

    // export_schema_with_title(
    //     &schema_for!(CosmosMsg<Empty>),
    //     &out_dir,
    //     "CosmosMsg_for_Empty",
    // );

    // export_schema_with_title(&schema_for!(AssetInfo), &out_dir, "AssetInfoBase_for_Addr");
}
