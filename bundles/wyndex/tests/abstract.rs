use cosmwasm_std::{coin, from_slice, Addr, Decimal, Uint128};

use wyndex::{
    asset::{AssetInfo, AssetInfoExt},
    factory::PartialStakeConfig,
};
use wyndex_stake::msg::{QueryMsg as StakeQueryMsg, ReceiveDelegationMsg, StakedResponse};
use wyndex_stake::state::Config as WyndexStakeConfig;

mod abstrct {
    use abstract_boot::Abstract;
    use boot_core::{prelude::instantiate_default_mock_env, Deploy};
    use cosmwasm_std::Empty;
    use wyndex::factory::{DefaultStakeConfig, DistributionFlow};
    use wyndex_bundle::{suite::SuiteBuilder, WynDex, WYNDEX_OWNER};

    use super::*;

    #[test]
    fn deploy() {
        let owner = Addr::unchecked(WYNDEX_OWNER);
        let (_state, mock) = instantiate_default_mock_env(&owner).unwrap();

        Abstract::deploy_on(mock.clone(), "1.0.0".parse().unwrap()).unwrap();

        let deployed = WynDex::deploy_on(mock.clone(), Empty {}).unwrap();
        let loaded = WynDex::load_from(mock.clone()).unwrap();

        assert_eq!(deployed, loaded)
    }
}
