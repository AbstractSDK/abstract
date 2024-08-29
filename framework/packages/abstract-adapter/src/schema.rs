use std::path::Path;

use abstract_sdk::base::{ExecuteEndpoint, InstantiateEndpoint, QueryEndpoint};
use abstract_std::{
    adapter,
    adapter::{AdapterExecuteMsg, AdapterQueryMsg},
};
use cosmwasm_schema::{export_schema_with_title, schema_for, write_api, QueryResponses};
use cosmwasm_std::Empty;
use schemars::JsonSchema;
use serde::Serialize;

use crate::{AdapterContract, AdapterError};

impl<
        Error: From<cosmwasm_std::StdError>
            + From<AdapterError>
            + From<abstract_sdk::AbstractSdkError>
            + From<abstract_std::AbstractError>
            + 'static,
        CustomExecMsg: Serialize + JsonSchema + AdapterExecuteMsg,
        CustomInitMsg: Serialize + JsonSchema,
        CustomQueryMsg: Serialize + JsonSchema + AdapterQueryMsg + QueryResponses,
        SudoMsg,
    > AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
{
    pub fn export_schema(out_dir: &Path) {
        write_api! {
            name: "schema",
            instantiate: adapter::InstantiateMsg<CustomInitMsg>,
            query: adapter::QueryMsg<CustomQueryMsg>,
            execute: adapter::ExecuteMsg<CustomExecMsg>,
            migrate: Empty,
        };

        // write out the module schema
        write_api! {
            name: "module-schema",
            instantiate: CustomInitMsg,
            query: CustomQueryMsg,
            execute: CustomExecMsg,
            migrate: Empty,
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
    }

    pub fn export_schema_custom<CustomExecuteHandle: JsonSchema>(out_dir: &Path) {
        let custom_execute_handle = schema_for!(CustomExecuteHandle);

        // Assert custom have base variant
        let base = schema_for!(adapter::BaseExecuteMsg);
        assert_have_endpoint(&custom_execute_handle, &base, "Base");

        // Assert custom have module variant
        let module = schema_for!(CustomExecMsg);
        assert_have_endpoint(&custom_execute_handle, &module, "Module");
        write_api! {
            name: "schema",
            instantiate: adapter::InstantiateMsg<CustomInitMsg>,
            query: adapter::QueryMsg<CustomQueryMsg>,
            execute: adapter::ExecuteMsg<CustomExecMsg>,
            migrate: Empty,
        };

        // write out the module schema
        write_api! {
            name: "module-schema",
            instantiate: CustomInitMsg,
            query: CustomQueryMsg,
            execute: CustomExecMsg,
            migrate: Empty,
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
