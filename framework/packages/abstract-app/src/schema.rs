use std::path::Path;

use abstract_std::app::{self, AppExecuteMsg, AppQueryMsg};
use cosmwasm_schema::{export_schema_with_title, schema_for, write_api, QueryResponses};
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    AppContract, AppError, ExecuteEndpoint, InstantiateEndpoint, MigrateEndpoint, QueryEndpoint,
};

impl<
        Error: From<cosmwasm_std::StdError>
            + From<AppError>
            + From<abstract_sdk::AbstractSdkError>
            + From<abstract_std::AbstractError>
            + 'static,
        CustomExecMsg: Serialize + JsonSchema + AppExecuteMsg,
        CustomInitMsg: Serialize + DeserializeOwned + JsonSchema,
        CustomQueryMsg: Serialize + JsonSchema + AppQueryMsg + QueryResponses,
        CustomMigrateMsg: Serialize + JsonSchema,
        SudoMsg: Serialize + JsonSchema,
    > AppContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, CustomMigrateMsg, SudoMsg>
{
    pub fn export_schema(out_dir: &Path) {
        write_api! {
            name: "schema",
            instantiate: app::InstantiateMsg<CustomInitMsg>,
            query: app::QueryMsg<CustomQueryMsg>,
            execute: app::ExecuteMsg<CustomExecMsg>,
            migrate: app::MigrateMsg<CustomMigrateMsg>,
        };

        // write out the module-specific schema
        write_api! {
            name: "module-schema",
            instantiate: CustomInitMsg,
            query: CustomQueryMsg,
            execute: CustomExecMsg,
            migrate: CustomMigrateMsg,
        };

        export_schema_with_title(
            &schema_for!(<Self as ExecuteEndpoint>::ExecuteMsg),
            out_dir,
            "ExecuteMsg",
        );
        export_schema_with_title(
            &schema_for!(<Self as InstantiateEndpoint>::InstantiateMsg),
            out_dir,
            "InstantiateMsg",
        );
        export_schema_with_title(
            &schema_for!(<Self as QueryEndpoint>::QueryMsg),
            out_dir,
            "QueryMsg",
        );
        export_schema_with_title(
            &schema_for!(<Self as MigrateEndpoint>::MigrateMsg),
            out_dir,
            "MigrateMsg",
        );
    }

    pub fn export_schema_custom<CustomExecuteHandle: JsonSchema>(out_dir: &Path) {
        let custom_execute_handle = schema_for!(CustomExecuteHandle);

        // Assert custom have base variant
        let base = schema_for!(app::BaseExecuteMsg);
        assert_have_endpoint(&custom_execute_handle, &base, "Base");

        // Assert custom have module variant
        let module = schema_for!(CustomExecMsg);
        assert_have_endpoint(&custom_execute_handle, &module, "Module");

        write_api! {
            name: "schema",
            instantiate: app::InstantiateMsg<CustomInitMsg>,
            query: app::QueryMsg<CustomQueryMsg>,
            execute: CustomExecuteHandle,
            migrate: app::MigrateMsg<CustomMigrateMsg>,
        };

        // write out the module-specific schema
        write_api! {
            name: "module-schema",
            instantiate: CustomInitMsg,
            query: CustomQueryMsg,
            execute: CustomExecMsg,
            migrate: CustomMigrateMsg,
        };

        export_schema_with_title(&custom_execute_handle, out_dir, "ExecuteMsg");
        export_schema_with_title(
            &schema_for!(<Self as InstantiateEndpoint>::InstantiateMsg),
            out_dir,
            "InstantiateMsg",
        );
        export_schema_with_title(
            &schema_for!(<Self as QueryEndpoint>::QueryMsg),
            out_dir,
            "QueryMsg",
        );
        export_schema_with_title(
            &schema_for!(<Self as MigrateEndpoint>::MigrateMsg),
            out_dir,
            "MigrateMsg",
        );
    }
}

fn assert_have_endpoint(
    custom: &schemars::schema::RootSchema,
    expected: &schemars::schema::RootSchema,
    variant_name: &str,
) {
    // Ensure we have requirements of app
    let variants = custom.schema.subschemas.clone().unwrap().one_of.unwrap();
    let expected_title = expected.schema.metadata.clone().unwrap().title.unwrap();

    let have_base = variants.iter().any(|schema| {
        if let schemars::schema::Schema::Object(schema) = schema {
            if let Some(object) = &schema.object {
                return object.required == schemars::Set::from([variant_name.to_lowercase()])
                    && object
                        .properties
                        .get(&variant_name.to_lowercase())
                        .cloned()
                        .unwrap()
                        .into_object()
                        .reference
                        .unwrap()
                        == format!("#/definitions/{expected_title}",);
            }
        }
        false
    });
    assert!(
        have_base,
        "Custom Execute variant must include {variant_name}({expected_title})"
    );
    let mut custom_definition = custom
        .definitions
        .get(&expected_title)
        .cloned()
        .unwrap()
        .into_object();
    // One of them can be without title
    custom_definition.metadata = expected.schema.metadata.clone();
    assert_eq!(
        custom_definition, expected.schema,
        "Custom Execute variant {variant_name}({expected_title}) have invalid type"
    );
}
