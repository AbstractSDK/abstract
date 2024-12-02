mod abstrct {
    use abstract_interface::Abstract;
    use cw_orch::prelude::*;
    use mockdex_bundle::WynDex;

    #[test]
    fn deploy() {
        let mock = MockBech32::new("mock");

        Abstract::deploy_on(mock.clone(), ()).unwrap();

        let deployed = WynDex::deploy_on(mock.clone(), Empty {}).unwrap();
        let loaded = WynDex::load_from(mock).unwrap();

        assert_eq!(deployed, loaded)
    }
}
