fn main() -> anyhow::Result<()> {
    // This is an integration test with Abstract And polytone EVM already deployed on Union

    // If it's not deployed, we can redeploy it here

    let chain = Daemon::builder().chain(UNION_TESTNET_8).build()?;
    let abs = Abstract::load_from(chain.clone())?;

    // We get the account and install the ICA client app on it

    // We start by sending some funds to the interchain account to be able to send it around in the ica action

    // We query the ICA client action from the script

    // We send the message from the account directly

    // We make sure the messages do the right actions with a query on the EVM chain
}
