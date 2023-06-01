use cosmwasm_std::Addr;

mod abstrct {
    use abstract_interface::Abstract;
    use cosmwasm_std::Empty;
    use cw_orch::deploy::Deploy;
    use cw_orch::mock::Mock;

    use wyndex_bundle::{WynDex, WYNDEX_OWNER};

    use super::*;

    #[test]
    fn deploy() {
        let owner = Addr::unchecked(WYNDEX_OWNER);
        let mock = Mock::new(&owner);

        Abstract::deploy_on(mock.clone(), Empty {}).unwrap();

        let deployed = WynDex::deploy_on(mock.clone(), Empty {}).unwrap();
        let loaded = WynDex::load_from(mock).unwrap();

        assert_eq!(deployed, loaded)
    }
}
