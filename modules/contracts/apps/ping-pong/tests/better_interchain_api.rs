use abstract_client::{AbstractClient, AbstractInterchainClient};
use cw_orch::anyhow;
use cw_orch::prelude::*;
use cw_orch_interchain::prelude::*;
pub const JUNO: &str = "juno-1";
pub const STARGAZE: &str = "stargaze-1";

#[test]
fn abstract_load_api() -> anyhow::Result<()> {
    // Start by deploying abstract completely
    let mock_interchain =
        MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);
    let interchain_abstract = AbstractInterchainClient::new(&mock_interchain)?;

    // Then we load abstract from state and make sure this is the same instance
    let juno_abstract = AbstractClient::new(mock_interchain.get_chain(JUNO)?)?;
    let stargaze_abstract = AbstractClient::new(mock_interchain.get_chain(STARGAZE)?)?;

    let loaded_interchain_abstract = AbstractInterchainClient::new(&mock_interchain)?;

    assert_eq!(
        interchain_abstract.client(JUNO)?.registry().address()?,
        juno_abstract.registry().address()?
    );

    assert_eq!(
        interchain_abstract.client(JUNO)?.registry().address()?,
        loaded_interchain_abstract
            .client(JUNO)?
            .registry()
            .address()?,
    );

    assert_eq!(
        interchain_abstract.client(STARGAZE)?.registry().address()?,
        stargaze_abstract.registry().address()?
    );
    assert_eq!(
        interchain_abstract.client(STARGAZE)?.registry().address()?,
        loaded_interchain_abstract
            .client(STARGAZE)?
            .registry()
            .address()?,
    );

    Ok(())
}
