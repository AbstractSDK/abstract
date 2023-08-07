use std::str::FromStr;

use crate::types::{
    ibc::MsgTransfer,
    token_factory::{MsgCreateDenom, MsgMint},
};
use anyhow::Result as AnyResult;
use cosmrs::{AccountId, Denom};
use cosmwasm_std::Coin;
use cw_orch::{
    daemon::DaemonError,
    interchain::{interchain_channel::InterchainChannel, IcResult},
    prelude::{interchain_channel_builder::InterchainChannelBuilder, Daemon, TxHandler},
    starship::Starship,
    state::ChainState,
};
use ibc_relayer_types::core::ics24_host::identifier::PortId;
use tokio::runtime::Runtime;

/// Creates a new denom using the token factory module.
/// This is used mainly for tests, but feel free to use that in production as well
pub async fn create_denom(daemon: &Daemon, token_name: &str) -> Result<(), DaemonError> {
    let creator = daemon.sender().to_string();
    daemon
        .wallet()
        .commit_tx(
            vec![MsgCreateDenom {
                sender: AccountId::from_str(creator.as_str())?,
                subdenom: token_name.to_string(),
            }],
            None,
        )
        .await?;

    log::info!("Created denom {}", get_denom(daemon, token_name));

    Ok(())
}

/// Gets the denom of a token created by a daemon object
/// This actually creates the denom for a token created by an address (which is here taken to be the daemon sender address)
/// This is mainly used for tests, but feel free to use that in production as well
pub fn get_denom(daemon: &Daemon, token_name: &str) -> String {
    let sender = daemon.sender().to_string();
    format!("factory/{}/{}", sender, token_name)
}

/// Mints new subdenom token for which the minter is the sender of Daemon object
/// This mints new tokens to the receiver address
/// This is mainly used for tests, but feel free to use that in production as well
pub async fn mint(
    daemon: &Daemon,
    receiver: &str,
    token_name: &str,
    amount: u128,
) -> Result<(), DaemonError> {
    let sender = daemon.sender().to_string();
    let denom = get_denom(daemon, token_name);

    daemon
        .wallet()
        .commit_tx(
            vec![MsgMint {
                sender: AccountId::from_str(sender.as_str())?,
                mint_to_address: AccountId::from_str(receiver)?,
                amount: Some(cosmrs::Coin {
                    denom: Denom::from_str(denom.as_str())?,
                    amount,
                }),
            }],
            None,
        )
        .await?;

    log::info!("Minted coins {} {}", amount, get_denom(daemon, token_name));

    Ok(())
}

// 1 hour should be sufficient for packet timeout
const TIMEOUT_IN_NANO_SECONDS: u64 = 3_600_000_000_000;

/// Ibc token transfer
/// This allows transfering token over a channel using an interchain_channel object
pub fn transfer_tokens(
    rt: &Runtime,
    origin: &Daemon,
    receiver: &str,
    fund: &Coin,
    ibc_channel: &InterchainChannel,
    memo: Option<String>,
) -> IcResult<()> {
    let chain_id = origin.state().0.chain_data.chain_id.to_string();

    let source_port = ibc_channel.get_chain(chain_id)?;

    // We send tokens using the ics20 message over the channel that is passed as an argument
    let send_tx = rt.block_on(origin.wallet().commit_tx(
        vec![MsgTransfer {
            source_port: source_port.port.to_string(),
            source_channel: source_port.channel.unwrap().to_string(),
            token: Some(cosmrs::Coin {
                amount: fund.amount.u128(),
                denom: Denom::from_str(fund.denom.as_str()).unwrap(),
            }),
            sender: AccountId::from_str(origin.sender().to_string().as_str()).unwrap(),
            receiver: AccountId::from_str(receiver).unwrap(),
            timeout_height: None,
            timeout_revision: None,
            timeout_timestamp: origin.block_info()?.time.nanos() + TIMEOUT_IN_NANO_SECONDS,
            memo,
        }],
        None,
    ))?;

    // We wait for the IBC tx to stop successfully
    rt.block_on(ibc_channel.await_ibc_execution(source_port.chain_id, send_tx.txhash))?;

    Ok(())
}

/* ####################### STARSHIP specific functions ########################### */

const ICS20_CHANNEL_VERSION: &str = "ics20-1";
/// Channel creation between the transfer channels of two blockchains of a starship integration
pub async fn create_transfer_channel(
    chain1: &str,
    chain2: &str,
    starship: &Starship,
) -> AnyResult<InterchainChannel> {
    let daemon_a = starship.daemon(chain1)?;
    let daemon_b = starship.daemon(chain2)?;
    Ok(InterchainChannelBuilder::default()
        .from_daemons(daemon_a, daemon_b)
        .port_a(PortId::transfer())
        .port_b(PortId::transfer())
        .create_channel(starship.client(), ICS20_CHANNEL_VERSION)
        .await?)
}
