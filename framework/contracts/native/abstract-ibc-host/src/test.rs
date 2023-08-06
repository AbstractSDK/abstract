use abstract_core::ibc_host::InstantiateMsg;
use abstract_core::ibc_host::QueryMsg;
use abstract_core::ibc_host::RegisteredChainResponse;
use abstract_core::objects::chain_name::ChainName;
use cosmwasm_std::from_binary;
use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::testing::mock_info;
use cosmwasm_std::IbcEndpoint;

use crate::contract::execute;
use crate::contract::instantiate;
use crate::contract::query;
use crate::ibc::receive_who_am_i;

#[test]
fn test_registered_client() {
    // Instantiate
    let mut deps = mock_dependencies();
    let info = mock_info("admin", &[]);
    instantiate(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        InstantiateMsg {
            account_factory_address: "dummy".to_string(),
            version_control_address: "foo".to_string(),
            ans_host_address: "bar".to_string(),
        },
    )
    .unwrap();

    // Register
    execute(
        deps.as_mut(),
        mock_env(),
        info,
        abstract_core::ibc_host::ExecuteMsg::RegisterChainClient {
            chain_id: "juno".to_string(),
            client: "juno-client".to_string(),
        },
    )
    .unwrap();

    // Query
    let client_name = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::AssociatedClient {
            chain: "juno".to_string(),
        },
    )
    .unwrap();
    let queried_client_name: RegisteredChainResponse = from_binary(&client_name).unwrap();
    assert_eq!(queried_client_name.client, "juno-client");

    receive_who_am_i(
        deps.as_mut(),
        "channel-1".to_string(),
        IbcEndpoint {
            channel_id: "channel-1".to_string(),
            port_id: "wasm.juno-client".to_string(),
        },
        ChainName::from("juno"),
        ChainName::from("osmosis-2"),
    )
    .unwrap();
}
