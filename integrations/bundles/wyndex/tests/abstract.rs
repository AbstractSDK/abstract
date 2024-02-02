mod abstrct {
    use abstract_interface::Abstract;
    use cosmwasm_std::{Addr, Empty};
    use cw_orch::prelude::*;
    use wyndex_bundle::{WynDex, WYNDEX_OWNER};

    #[test]
    fn deploy() {
        let owner = Addr::unchecked(WYNDEX_OWNER);
        let mock = Mock::new(&owner);

        Abstract::deploy_on(mock.clone(), owner.to_string()).unwrap();

        let deployed = WynDex::deploy_on(mock.clone(), Empty {}).unwrap();
        let loaded = WynDex::load_from(mock).unwrap();

        assert_eq!(deployed, loaded)
    }
}
