#[cfg(feature = "daemon")]
#[test]
fn test_deploy_abstract() {
    use abstract_core::ans_host::InstantiateMsg;
    use cw_orch::daemon::DaemonError;
    use cw_orch::deploy::Deploy;
    use std::env::set_var;

    use abstract_interface::Abstract;
    use cw_orch::prelude::*;

    set_var("TEST_MNEMONIC","extra infant liquid afraid lens legend frown horn flame vessel palm nuclear jazz build iron squeeze review stock they snake dawn metal outdoor muffin");

    let runtime = tokio::runtime::Runtime::new().unwrap();

    let daemon = Daemon::builder()
        .chain(networks::osmosis::OSMO_4)
        .handle(runtime.handle())
        .build()
        .unwrap();

    let abstr = Abstract::load_from(daemon).unwrap();

    // We test if the wasm file is present alright
    abstr.ans_host.wasm();
    // Now we upload abstract using the file loaded configuration

    let error = abstr
        .ans_host
        .instantiate(&InstantiateMsg {}, None, None)
        .unwrap_err();

    // We expect the error to be that the account doesn't exist
    match &error {
        CwOrchError::DaemonError(DaemonError::Status(s)) => {
            if s.message()
                .ne("account osmo1cjh5gskpmvd5nx2hdeupdz0wadm78mxl4l3cl0 not found")
            {
                panic!("Error not expected, {:?}", error);
            }
        }
        _ => panic!("Error not expected, {:?}", error),
    }
}
