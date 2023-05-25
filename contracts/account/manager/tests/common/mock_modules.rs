#![allow(dead_code)]

use abstract_adapter::gen_adapter_mock;
use abstract_adapter::mock::MockInitMsg;
use abstract_adapter::{mock::MockError as AdapterMockError, AdapterContract};
use abstract_app::gen_app_mock;
use abstract_app::mock::MockError as AppMockError;
use abstract_app::AppContract;
use abstract_core::objects::dependency::StaticDependency;
use abstract_interface::{AdapterDeployer, AppDeployer};
use cw_orch::prelude::*;

pub type MockAdapterContract = AdapterContract<AdapterMockError, Empty, Empty, Empty, Empty, Empty>;
pub type MockAppContract = AppContract<AppMockError, Empty, Empty, Empty, Empty, Empty, Empty>;

pub use self::adapter_1::*;
pub use self::adapter_2::*;
pub use self::app_1::*;

pub const V1: &str = "1.0.0";
pub const V2: &str = "2.0.0";

/// deploys different version adapters and app for migration testing
pub fn deploy_modules(mock: &Mock) {
    self::BootMockAdapter1V1::new_test(mock.clone())
        .deploy(V1.parse().unwrap(), MockInitMsg)
        .unwrap();

    // do same for version 2
    self::BootMockAdapter1V2::new_test(mock.clone())
        .deploy(V2.parse().unwrap(), MockInitMsg)
        .unwrap();

    // and now for adapter 2
    self::BootMockAdapter2V1::new_test(mock.clone())
        .deploy(V1.parse().unwrap(), MockInitMsg)
        .unwrap();

    // do same for version 2
    self::BootMockAdapter2V2::new_test(mock.clone())
        .deploy(V2.parse().unwrap(), MockInitMsg)
        .unwrap();

    // and now for app 1
    self::BootMockApp1V1::new_test(mock.clone())
        .deploy(V1.parse().unwrap())
        .unwrap();

    // do same for version 2
    self::BootMockApp1V2::new_test(mock.clone())
        .deploy(V2.parse().unwrap())
        .unwrap();
}

pub mod adapter_1 {
    use super::*;

    pub const MOCK_ADAPTER_ID: &str = "tester:mock-adapter1";

    pub use self::v1::*;
    pub use self::v2::*;

    pub mod v1 {
        use super::*;
        gen_adapter_mock!(BootMockAdapter1V1, MOCK_ADAPTER_ID, "1.0.0", &[]);
    }

    pub mod v2 {
        use super::*;
        gen_adapter_mock!(BootMockAdapter1V2, MOCK_ADAPTER_ID, "2.0.0", &[]);
    }
}

pub mod adapter_2 {
    use super::*;

    pub const MOCK_ADAPTER_ID: &str = "tester:mock-adapter2";

    pub use self::v0_1_0::*;
    pub use self::v1::*;
    pub use self::v2_0_0::*;

    pub mod v1 {
        use super::*;
        gen_adapter_mock!(BootMockAdapter2V1, MOCK_ADAPTER_ID, "1.0.0", &[]);
    }

    pub mod v2_0_0 {
        use super::*;
        gen_adapter_mock!(BootMockAdapter2V2, MOCK_ADAPTER_ID, "2.0.0", &[]);
    }

    pub mod v0_1_0 {
        use super::*;
        gen_adapter_mock!(BootMockAdapter2V0_1_0, MOCK_ADAPTER_ID, "0.1.0", &[]);
    }
}

// app 1 depends on adapter 1 and adapter 2
pub mod app_1 {
    use super::*;
    pub use v1::*;
    pub use v2::*;
    pub const MOCK_APP_ID: &str = "tester:mock-app1";
    pub mod v1 {
        use super::*;
        gen_app_mock!(
            BootMockApp1V1,
            MOCK_APP_ID,
            "1.0.0",
            &[
                StaticDependency::new(adapter_1::MOCK_ADAPTER_ID, &[V1]),
                StaticDependency::new(adapter_2::MOCK_ADAPTER_ID, &[V1]),
            ]
        );
    }

    pub mod v2 {
        use super::*;
        gen_app_mock!(
            BootMockApp1V2,
            MOCK_APP_ID,
            "2.0.0",
            &[
                StaticDependency::new(adapter_1::MOCK_ADAPTER_ID, &[V2]),
                StaticDependency::new(adapter_2::MOCK_ADAPTER_ID, &[V2]),
            ]
        );
    }
}
