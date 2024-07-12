mod abstrct {
    use abstract_interface::Abstract;
    use cw_orch::prelude::*;
    use wyndex_bundle::WynDex;

    #[test]
    fn deploy() {
        let mock = MockBech32::new("mock");
        let owner = mock.sender_addr();

        Abstract::deploy_on(mock.clone(), owner.to_string()).unwrap();

        let deployed = WynDex::deploy_on(mock.clone(), Empty {}).unwrap();
        let loaded = WynDex::load_from(mock).unwrap();

        assert_eq!(deployed, loaded)
    }
}
