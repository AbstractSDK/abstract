use cosmwasm_std::Coin;

use osmosis_std_cosmwasm_test::msg::InstantiateMsg;
use osmosis_test_tube::{Gamm, Module, OsmosisTestApp, SigningAccount, Wasm};
use std::path::PathBuf;

pub fn with_env_setup(
    run: impl Fn(&OsmosisTestApp, Wasm<OsmosisTestApp>, SigningAccount, u64, String),
    debug: bool,
) {
    let app = OsmosisTestApp::new();
    let wasm = Wasm::new(&app);
    let signer = app
        .init_account(&[
            Coin::new(100_000_000_000, "uosmo"),
            Coin::new(100_000_000_000, "uion"),
            Coin::new(100_000_000_000, "uusdc"),
            Coin::new(100_000_000_000, "uiou"),
        ])
        .unwrap();

    let code_id = wasm
        .store_code(&get_wasm_byte_code(), None, &signer)
        .unwrap()
        .data
        .code_id;
    let contract_addr = wasm
        .instantiate(code_id, &InstantiateMsg { debug }, None, None, &[], &signer)
        .unwrap()
        .data
        .address;
    run(&app, wasm, signer, code_id, contract_addr)
}

pub fn mock_balancner_pool() -> osmosis_std::types::osmosis::gamm::v1beta1::Pool {
    osmosis_std::types::osmosis::gamm::v1beta1::Pool {
        address: "osmo1mw0ac6rwlp5r8wapwk3zs6g29h8fcscxqakdzw9emkne6c8wjp9q0t3v8t".to_string(),
        id: 1,
        pool_params: Some(osmosis_std::types::osmosis::gamm::v1beta1::PoolParams {
            swap_fee: "0.010000000000000000".to_string(),
            exit_fee: "0.010000000000000000".to_string(),
            smooth_weight_change_params: None,
        }),
        future_pool_governor: "".to_string(),
        total_shares: Some(osmosis_std::types::cosmos::base::v1beta1::Coin {
            denom: "gamm/pool/1".to_string(),
            amount: "100000000000000000000".to_string(),
        }),
        pool_assets: vec![
            osmosis_std::types::osmosis::gamm::v1beta1::PoolAsset {
                token: Some(osmosis_std::types::cosmos::base::v1beta1::Coin {
                    denom: "uion".to_string(),
                    amount: "1000".to_string(),
                }),
                weight: "1073741824000000".to_string(),
            },
            osmosis_std::types::osmosis::gamm::v1beta1::PoolAsset {
                token: Some(osmosis_std::types::cosmos::base::v1beta1::Coin {
                    denom: "uosmo".to_string(),
                    amount: "1000".to_string(),
                }),
                weight: "1073741824000000".to_string(),
            },
        ],
        total_weight: "2147483648000000".to_string(),
    }
}

pub fn setup_pools(app: &OsmosisTestApp, signer: &SigningAccount) -> Vec<u64> {
    let gamm = Gamm::new(app);

    // resulted in `mock_balancner_pool`
    let balancer_pool_id = gamm
        .create_basic_pool(
            &[Coin::new(1_000, "uosmo"), Coin::new(1_000, "uion")],
            signer,
        )
        .unwrap()
        .data
        .pool_id;

    vec![balancer_pool_id]
}
pub fn get_wasm_byte_code() -> Vec<u8> {
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    std::fs::read(
        manifest_path
            .join("..")
            .join("..")
            .join("target")
            .join("wasm32-unknown-unknown")
            .join("release")
            .join("osmosis_std_cosmwasm_test.wasm"),
    )
    .unwrap()
}
