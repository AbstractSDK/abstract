pub mod abstract_ibc;

use cw_orch::environment::ChainKind;
use cw_orch::prelude::{
    networks::{HARPOON_4, JUNO_1, OSMO_5, PHOENIX_1, PION_1, PISCO_1, UNI_6},
    *,
};
use serde::{Deserialize, Serialize};

const GAS_TO_DEPLOY: u64 = 60_000_000;
pub const SUPPORTED_CHAINS: &[ChainInfo] = &[
    UNI_6, OSMO_5, PISCO_1, PHOENIX_1, JUNO_1, PION_1, NEUTRON_1, HARPOON_4,
];

pub const NEUTRON_1: ChainInfo = ChainInfo {
    kind: ChainKind::Mainnet,
    chain_id: "neutron-1",
    gas_denom: "untrn",
    gas_price: 0.001,
    grpc_urls: &["https://grpc.novel.remedy.tm.p2p.org"],
    network_info: networks::neutron::NEUTRON_NETWORK,
    lcd_url: Some("https://rest-kralum.neutron-1.neutron.org"),
    fcd_url: None,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeploymentStatus {
    pub chain_ids: Vec<String>,
    pub success: bool,
}

pub async fn assert_wallet_balance<'a>(mut chains: Vec<ChainInfo>) -> Vec<ChainInfo> {
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

        let gas_denom = chain.state.as_ref().chain_data.gas_denom.clone();
        let gas_price = chain.state.as_ref().chain_data.gas_price;
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
        if fee > balance.amount.parse().unwrap() {
            panic!("Not enough funds on chain {} to deploy the contract. Needed: {}{} but only have: {}{}", chain_info.chain_id, fee, gas_denom, balance.amount, gas_denom);
        }
        // check if we have enough funds
    }

    chains
}
