fn main() -> anyhow::Result<()> {
    // This is an integration test with Abstract And polytone EVM already deployed on Union

    // If it's not deployed, we can redeploy it here

    let chain = Daemon::builder().chain(UNION_TESTNET_8).build()?;
    let abs = Abstract::load_from(chain.clone())?;
}
