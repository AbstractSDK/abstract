pub mod abstract_ibc;

use cw_orch::daemon::networks::neutron::NEUTRON_NETWORK;
use cw_orch::environment::{ChainKind, NetworkInfo};
use cw_orch::prelude::{
    networks::{HARPOON_4, JUNO_1, OSMO_5, PHOENIX_1, PION_1, PISCO_1, UNI_6},
    *,
};
use serde::{Deserialize, Serialize};

const GAS_TO_DEPLOY: u64 = 60_000_000;
pub const SUPPORTED_CHAINS: &[ChainInfo] = &[
    UNI_6, OSMO_5, PISCO_1, PHOENIX_1, JUNO_1, PION_1, NEUTRON_1, HARPOON_4,
];

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeploymentStatus {
    pub chain_ids: Vec<String>,
    pub success: bool,
}

pub async fn assert_wallet_balance(mut chains: Vec<ChainInfoOwned>) -> Vec<ChainInfoOwned> {
    if chains.is_empty() {
        chains = SUPPORTED_CHAINS.iter().cloned().map(Into::into).collect();
    }
    // check that the wallet has enough gas on all the chains we want to support
    for chain_info in &chains {
        let chain = DaemonAsyncBuilder::default()
            .chain(chain_info.clone())
            .build()
            .await
            .unwrap();

        let gas_denom = chain.state.chain_data.gas_denom.clone();
        let gas_price = chain.state.chain_data.gas_price;
        let fee = (GAS_TO_DEPLOY as f64 * gas_price) as u128;
        let bank = queriers::Bank::new_async(chain.channel());
        let balance = bank
            ._balance(chain.sender(), Some(gas_denom.clone()))
            .await
            .unwrap()
            .clone()[0]
            .clone();

        log::debug!(
            "Checking balance {} on chain {}, address {}. Expecting {}{}",
            balance.amount,
            chain_info.chain_id,
            chain.sender(),
            fee,
            gas_denom
        );
        if fee > balance.amount.u128() {
            panic!("Not enough funds on chain {} to deploy the contract. Needed: {}{} but only have: {}{}", chain_info.chain_id, fee, gas_denom, balance.amount, gas_denom);
        }
        // check if we have enough funds
    }

    chains
}

pub const ROLLKIT_NETWORK: NetworkInfo = NetworkInfo {
    chain_name: "rosm",
    pub_address_prefix: "wasm",
    coin_type: 118u32,
};

const ROLLKIT_GRPC: &str = "http://grpc.rosm.rollkit.dev:9290";
pub const ROLLKIT_TESTNET: ChainInfo = ChainInfo {
    kind: ChainKind::Testnet,
    chain_id: "rosm",
    gas_denom: "urosm",
    gas_price: 0.025,
    grpc_urls: &[ROLLKIT_GRPC],
    network_info: ROLLKIT_NETWORK,
    lcd_url: None,
    fcd_url: None,
};

/// <https://github.com/cosmos/chain-registry/blob/master/neutron/chain.json>
pub const NEUTRON_1: ChainInfo = ChainInfo {
    kind: ChainKind::Mainnet,
    chain_id: "neutron-1",
    gas_denom: "untrn",
    gas_price: 0.075,
    grpc_urls: &["http://grpc-kralum.neutron-1.neutron.org:80"],
    network_info: NEUTRON_NETWORK,
    lcd_url: Some("https://rest-kralum.neutron-1.neutron.org"),
    fcd_url: None,
};
