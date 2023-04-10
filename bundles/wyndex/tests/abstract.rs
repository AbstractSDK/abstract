use cosmwasm_std::Addr;

mod abstrct {
    use abstract_boot::Abstract;
    use abstract_boot::boot_core::{instantiate_default_mock_env, Deploy};
    use cosmwasm_std::Empty;

    use wyndex_bundle::{WynDex, WYNDEX_OWNER};

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
