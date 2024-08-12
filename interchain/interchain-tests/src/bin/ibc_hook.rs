// This script is used for testing a connection between 2 chains
// This script sets up tokens and channels between transfer ports to transfer those tokens
// This also mints tokens to the chain sender for future interactions

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use abstract_interchain_tests::{setup::set_starship_env, JUNO};
use abstract_interface::{connection::connect_one_way_to, Abstract};
use abstract_sdk::{IbcHooksBuilder, IbcMemoBuilder};
use abstract_std::{
    ans_host::ExecuteMsgFns,
    objects::{TruncatedChainId, UncheckedChannelEntry},
    ICS20, PROXY,
};
use anyhow::Result as AnyResult;
use cosmwasm_std::{coin, coins};
use cw_orch::{daemon::RUNTIME, prelude::*};
use cw_orch_interchain::prelude::*;
use cw_orch_proto::tokenfactory::{create_denom, get_denom, mint};
use ibc_relayer_types::core::ics24_host::identifier::PortId;

// Note: Truncated chain id have to be different
pub const JUNO2: &str = "junotwo-1";

pub fn test_pfm() -> AnyResult<()> {
    dotenv::dotenv().ok();
    set_starship_env();
    env_logger::init();

    let starship = Starship::new(None).unwrap();
    let interchain = starship.interchain_env();

    let juno = interchain.get_chain(JUNO).unwrap();
    let juno2 = interchain.get_chain(JUNO2).unwrap();

    // Create a channel between the 2 chains for the transfer ports
    // JUNO>JUNO2
    let juno_juno2_channel = interchain
        .create_channel(
            JUNO,
            JUNO2,
            &PortId::transfer(),
            &PortId::transfer(),
            "ics20-1",
            Some(cosmwasm_std::IbcOrder::Unordered),
        )?
        .interchain_channel;

    let abstr_juno = Abstract::deploy_on(juno.clone(), juno.sender_addr().to_string())?;
    let abstr_juno2 = Abstract::deploy_on(juno2.clone(), juno2.sender_addr().to_string())?;
    connect_one_way_to(&abstr_juno, &abstr_juno2, &interchain)?;

    // Faster to load if deployed
    // let abstr_juno = Abstract::load_from(juno.clone())?;
    // let abstr_juno2 = Abstract::load_from(juno2.clone())?;

    let counter_juno2 = counter_different_cw_orch::init_counter(juno2.clone())?;

    let sender = juno.sender_addr().to_string();

    let test_amount: u128 = 100_000_000_000;
    let token_subdenom = format!(
        "testtoken{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    // Create Denom
    create_denom(&juno, token_subdenom.as_str())?;

    // Mint Denom
    mint(&juno, sender.as_str(), token_subdenom.as_str(), test_amount)?;

    // Register this channel with the abstract ibc implementation for sending tokens
    abstr_juno.ans_host.update_channels(
        vec![(
            UncheckedChannelEntry {
                connected_chain: TruncatedChainId::from_chain_id(JUNO2).to_string(),
                protocol: ICS20.to_string(),
            },
            juno_juno2_channel
                .get_chain(JUNO)?
                .channel
                .unwrap()
                .to_string(),
        )],
        vec![],
    )?;

    // Create a test account + Remote account

    let origin_account = abstr_juno.account_factory.create_default_account(
        abstract_client::GovernanceDetails::Monarchy {
            monarch: juno.sender_addr().to_string(),
        },
    )?;
    origin_account.manager.set_ibc_status(true)?;

    // Send funds to the remote account
    RUNTIME.block_on(juno.sender().bank_send(
        &origin_account.proxy.addr_str()?,
        vec![coin(test_amount, get_denom(&juno, token_subdenom.as_str()))],
    ))?;

    let memo = IbcHooksBuilder::new(
        counter_juno2.addr_str()?,
        &counter_contract::msg::ExecuteMsg::Increment {},
    )
    .build()?;
    // We send from osmosis to juno funds with pfm memo that includes juno-stargaze channel
    origin_account.manager.execute_on_module(
        PROXY,
        abstract_std::proxy::ExecuteMsg::IbcAction {
            msg: abstract_std::ibc_client::ExecuteMsg::SendFunds {
                host_chain: TruncatedChainId::from_chain_id(JUNO2),
                funds: coins(10_000_000_000, get_denom(&juno, token_subdenom.as_str())),
                memo: Some(memo.clone()),
                receiver: Some(counter_juno2.addr_str()?),
            },
        },
    )?;
    origin_account.manager.execute_on_module(
        PROXY,
        abstract_std::proxy::ExecuteMsg::IbcAction {
            msg: abstract_std::ibc_client::ExecuteMsg::SendFunds {
                host_chain: TruncatedChainId::from_chain_id(JUNO2),
                funds: coins(10_000_000_000, get_denom(&juno, token_subdenom.as_str())),
                memo: Some(memo.clone()),
                receiver: Some(counter_juno2.addr_str()?),
            },
        },
    )?;
    origin_account.manager.execute_on_module(
        PROXY,
        abstract_std::proxy::ExecuteMsg::IbcAction {
            msg: abstract_std::ibc_client::ExecuteMsg::SendFunds {
                host_chain: TruncatedChainId::from_chain_id(JUNO2),
                funds: coins(10_000_000_000, get_denom(&juno, token_subdenom.as_str())),
                memo: Some(memo),
                receiver: Some(counter_juno2.addr_str()?),
            },
        },
    )?;
    log::info!("waiting for ibc_hook to finish tx");
    std::thread::sleep(Duration::from_secs(15));

    let count_juno2 = counter_juno2.query(&counter_contract::msg::QueryMsg::GetCount {})?;
    log::info!("count juno2: {count_juno2:?}");

    // Verify the funds have been received
    let count_juno2_balance = juno2
        .bank_querier()
        .balance(counter_juno2.addr_str()?, None)?;

    log::info!("count_juno2 balance, {:?}", count_juno2_balance);
    Ok(())
}

// Counter contract doesn't use crates.io version of cw-orch, some weird hacks here.
mod counter_different_cw_orch {
    use cw_orch::interface;
    use cw_orch::prelude::*;

    #[interface(
        counter_contract::msg::InstantiateMsg,
        counter_contract::msg::ExecuteMsg,
        counter_contract::msg::QueryMsg,
        counter_contract::msg::MigrateMsg,
        id = "counter_contract"
    )]
    pub struct CounterContract;

    impl<Chain> Uploadable for CounterContract<Chain> {
        fn wasm(chain: &ChainInfoOwned) -> WasmPath {
            let chain = chain.clone();
            let kind = match chain.kind {
                networks::ChainKind::Local => cw_orch_counter::environment::ChainKind::Local,
                networks::ChainKind::Mainnet => cw_orch_counter::environment::ChainKind::Mainnet,
                networks::ChainKind::Testnet => cw_orch_counter::environment::ChainKind::Testnet,
                networks::ChainKind::Unspecified => unreachable!(),
            };
            let chain = cw_orch_counter::prelude::ChainInfoOwned {
                chain_id: chain.chain_id,
                gas_denom: chain.gas_denom,
                gas_price: chain.gas_price,
                grpc_urls: chain.grpc_urls,
                lcd_url: chain.lcd_url,
                fcd_url: chain.fcd_url,
                network_info: cw_orch_counter::environment::NetworkInfoOwned {
                    chain_name: chain.network_info.chain_name,
                    pub_address_prefix: chain.network_info.pub_address_prefix,
                    coin_type: chain.network_info.coin_type,
                },
                kind,
            };

            let wasm_path =
                <counter_contract::CounterContract<Chain> as cw_orch_counter::prelude::Uploadable>::wasm(
                    &chain,
                );
            WasmPath::new(wasm_path.path()).unwrap()
        }

        fn wrapper() -> Box<dyn MockContract<Empty, Empty>> {
            Box::new(
                ContractWrapper::new_with_empty(
                    counter_contract::contract::execute,
                    counter_contract::contract::instantiate,
                    counter_contract::contract::query,
                )
                .with_migrate(counter_contract::contract::migrate),
            )
        }
    }

    pub fn init_counter<Chain: CwEnv>(chain: Chain) -> anyhow::Result<CounterContract<Chain>> {
        let counter = CounterContract::new(chain);
        counter.upload()?;
        counter.instantiate(
            &counter_contract::msg::InstantiateMsg { count: 0 },
            None,
            None,
        )?;
        Ok(counter)
    }
}

pub fn main() {
    test_pfm().unwrap();
}
