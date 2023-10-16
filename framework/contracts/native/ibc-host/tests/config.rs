use abstract_core::ibc_host::InstantiateMsg;
use abstract_core::ibc_host::QueryMsg;
use abstract_core::ibc_host::ClientProxyResponse;

use abstract_ibc_host::contract::execute;
use abstract_ibc_host::contract::instantiate;
use abstract_ibc_host::contract::query;
use cosmwasm_std::from_binary;
use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::testing::mock_info;

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
        abstract_core::ibc_host::ExecuteMsg::RegisterChainProxy {
            chain: "juno".into(),
            proxy: "juno-proxy".to_string(),
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
    let queried_client_name: ClientProxyResponse = from_binary(&client_name).unwrap();
    assert_eq!(queried_client_name.proxy, "juno-proxy");
}
