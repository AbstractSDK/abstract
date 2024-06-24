use abstract_interface::connection::abstract_ibc_connection_with;
use abstract_interface::Abstract;
use anyhow::Result as AnyResult;
use cw_orch::prelude::*;
use cw_orch_interchain::prelude::*;
use cw_orch_polytone::Polytone;
use polytone::handshake::POLYTONE_VERSION;

pub fn ibc_connect_abstract<Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>>(
    interchain: &IBC,
    origin_chain_id: &str,
    remote_chain_id: &str,
) -> AnyResult<(Abstract<Chain>, Abstract<Chain>)> {
    let origin_chain = interchain.chain(origin_chain_id).unwrap();
    let remote_chain = interchain.chain(remote_chain_id).unwrap();

    // Deploying abstract and the IBC abstract logic
    let abstr_origin = Abstract::load_from(origin_chain.clone())?;
    let abstr_remote = Abstract::load_from(remote_chain.clone())?;

    // Deploying polytone on both chains
    Polytone::deploy_on(origin_chain.clone(), None)?;
    Polytone::deploy_on(remote_chain.clone(), None)?;

    ibc_connect_polytone_and_abstract(interchain, origin_chain_id, remote_chain_id)?;

    Ok((abstr_origin, abstr_remote))
}

pub fn ibc_abstract_setup<Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>>(
    interchain: &IBC,
    origin_chain_id: &str,
    remote_chain_id: &str,
) -> AnyResult<(Abstract<Chain>, Abstract<Chain>)> {
    let origin_chain = interchain.chain(origin_chain_id).unwrap();
    let remote_chain = interchain.chain(remote_chain_id).unwrap();

    // Deploying abstract and the IBC abstract logic
    let abstr_origin =
        Abstract::deploy_on(origin_chain.clone(), origin_chain.sender().to_string())?;
    let abstr_remote =
        Abstract::deploy_on(remote_chain.clone(), remote_chain.sender().to_string())?;

    // Deploying polytone on both chains
    Polytone::deploy_on(origin_chain.clone(), None)?;
    Polytone::deploy_on(remote_chain.clone(), None)?;

    ibc_connect_polytone_and_abstract(interchain, origin_chain_id, remote_chain_id)?;

    Ok((abstr_origin, abstr_remote))
}

pub fn ibc_connect_polytone_and_abstract<Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>>(
    interchain: &IBC,
    origin_chain_id: &str,
    remote_chain_id: &str,
) -> AnyResult<()> {
    let origin_chain = interchain.chain(origin_chain_id).unwrap();
    let remote_chain = interchain.chain(remote_chain_id).unwrap();

    let abstr_origin = Abstract::load_from(origin_chain.clone())?;
    let abstr_remote = Abstract::load_from(remote_chain.clone())?;

    let origin_polytone = Polytone::load_from(origin_chain.clone())?;
    let remote_polytone = Polytone::load_from(remote_chain.clone())?;

    // Creating a connection between 2 polytone deployments
    interchain.create_contract_channel(
        &origin_polytone.note,
        &remote_polytone.voice,
        POLYTONE_VERSION,
        None, // Unordered channel
    )?;
    // Create the connection between client and host
    abstract_ibc_connection_with(&abstr_origin, interchain, &abstr_remote, &origin_polytone)?;
    Ok(())
}
