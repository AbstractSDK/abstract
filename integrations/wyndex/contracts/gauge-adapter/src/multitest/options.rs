use cosmwasm_std::Addr;
use wyndex::asset::AssetInfo;

use crate::multitest::suite::SuiteBuilder;

#[test]
fn option_queries() {
    let mut suite = SuiteBuilder::new().build();

    // create pairs to reward
    let (pair1_staking, _) = suite
        .create_pair_staking(
            AssetInfo::Native("juno".to_string()),
            AssetInfo::Native("asdf".to_string()),
        )
        .unwrap();
    let (pair2_staking, _) = suite
        .create_pair_staking(
            AssetInfo::Native("juno".to_string()),
            AssetInfo::Native("test".to_string()),
        )
        .unwrap();

    // get all options
    let options = suite.query_all_options().unwrap();
    assert_eq!(
        vec![pair1_staking.0.to_string(), pair2_staking.0.to_string()],
        options
    );

    // check option validity
    assert!(suite
        .query_check_option(pair1_staking.0.to_string())
        .unwrap());
    assert!(suite
        .query_check_option(pair2_staking.0.to_string())
        .unwrap());
    assert!(!suite
        .query_check_option(Addr::unchecked("invalid").to_string())
        .unwrap());
}
