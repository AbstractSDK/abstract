use abstract_interface::connection::connect_one_way_to;
use abstract_interface::*;
use abstract_std::{
    account,
    objects::{gov_type::GovernanceDetails, TruncatedChainId, UncheckedChannelEntry},
    ACCOUNT, ICS20,
};
use cosmwasm_std::coin;
use cw_orch::prelude::*;
use cw_orch_interchain::prelude::*;

type AResult = anyhow::Result<()>; // alias for Result<(), anyhow::Error>

pub const SOURCE_CHAIN_ID: &str = "source-1";
pub const DEST_CHAIN_ID: &str = "dest-1";

#[test]
fn transfer_without_messages() -> AResult {
    let interchain =
        MockBech32InterchainEnv::new(vec![(SOURCE_CHAIN_ID, "source"), (DEST_CHAIN_ID, "dest")]);

    let src = interchain.get_chain(SOURCE_CHAIN_ID)?;
    let dst = interchain.get_chain(DEST_CHAIN_ID)?;

    let src_abstr = Abstract::deploy_on_mock(src.clone())?;
    let dest_abstr = Abstract::deploy_on_mock(dst.clone())?;

    connect_one_way_to(
        &src_abstr.call_as(&Abstract::mock_admin(&src)),
        &dest_abstr.call_as(&Abstract::mock_admin(&dst)),
        &interchain,
    )?;

    let ics20_channel = interchain
        .create_channel(
            SOURCE_CHAIN_ID,
            DEST_CHAIN_ID,
            &PortId::transfer(),
            &PortId::transfer(),
            "ics20-1",
            Some(cosmwasm_std::IbcOrder::Unordered),
        )?
        .interchain_channel;

    // Update the channels on the src side (for sending tokens)
    src_abstr
        .ans_host
        .call_as(&Abstract::mock_admin(&src))
        .update_channels(
            vec![(
                UncheckedChannelEntry {
                    connected_chain: TruncatedChainId::from_chain_id(DEST_CHAIN_ID).to_string(),
                    protocol: ICS20.to_string(),
                },
                ics20_channel
                    .get_chain(SOURCE_CHAIN_ID)?
                    .channel
                    .unwrap()
                    .to_string(),
            )],
            vec![],
        )?;

    // Update the channels on the src side (for sending back tokens)
    dest_abstr
        .ans_host
        .call_as(&Abstract::mock_admin(&dst))
        .update_channels(
            vec![(
                UncheckedChannelEntry {
                    connected_chain: TruncatedChainId::from_chain_id(SOURCE_CHAIN_ID).to_string(),
                    protocol: ICS20.to_string(),
                },
                ics20_channel
                    .get_chain(DEST_CHAIN_ID)?
                    .channel
                    .unwrap()
                    .to_string(),
            )],
            vec![],
        )?;

    let account = AccountI::new(ACCOUNT, src.clone());

    account.instantiate(
        &account::InstantiateMsg {
            name: Some(String::from("first_account")),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: None,
            install_modules: vec![],
            account_id: None,
            owner: GovernanceDetails::Monarchy {
                monarch: src.sender_addr().to_string(),
            },
            authenticator: None,
        },
        None,
        &[],
    )?;
    account.set_ibc_status(true)?;

    let funds_to_transfer = coin(100_000, "usource");

    src.add_balance(&account.address()?, vec![funds_to_transfer.clone()])?;

    let tx_response = account.send_funds_with_actions(
        vec![],
        funds_to_transfer.clone(),
        TruncatedChainId::from_chain_id(&dst.chain_id()),
    )?;

    interchain.await_and_check_packets(SOURCE_CHAIN_ID, tx_response)?;

    let src_account_balance = src.balance(&account.address()?, None)?;
    assert!(src_account_balance.is_empty());

    let dst_host_balance = dst.balance(&dest_abstr.ibc.host.address()?, None)?;
    assert!(dst_host_balance.is_empty());

    // We fetch the remote account balance
    let mut remote_account_id = account.id()?;
    remote_account_id.push_chain(TruncatedChainId::from_chain_id(SOURCE_CHAIN_ID));

    let remote_account = AccountI::load_from(&dest_abstr, remote_account_id)?;

    let dest_account_balance = dst.balance(&remote_account.address()?, None)?;
    assert_eq!(dest_account_balance.len(), 1);

    Ok(())
}
