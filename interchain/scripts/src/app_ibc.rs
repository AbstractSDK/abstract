#[test]
fn test_adapter_ibc() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");

    // We use the abstract client to deploy abstract and then a mock app
    let interchain =
        MockBech32InterchainEnv::new(vec![("juno-1", "juno"), ("osmosis-1", "osmosis")])?;

    let juno = interchain.chain("juno-1");
    let osmosis = interchain.chain("osmosis-1");

    // We test ibc on this mock app

    Ok(())
}
