#![allow(dead_code)]

use abstract_api::gen_api_mock;
use abstract_api::mock::MockInitMsg;
use abstract_api::{mock::MockError as ApiMockError, ApiContract};
use abstract_app::gen_app_mock;
use abstract_app::mock::MockError as AppMockError;
use abstract_app::AppContract;
use abstract_boot::{ApiDeployer, AppDeployer};
use abstract_core::objects::dependency::StaticDependency;
// use boot_core::{ContractWrapper};
use boot_core::{Empty, Mock};

pub type MockApiContract = ApiContract<ApiMockError, Empty, Empty, Empty, Empty, Empty>;
pub type MockAppContract = AppContract<AppMockError, Empty, Empty, Empty, Empty, Empty, Empty>;
pub use self::api_1::*;
pub use self::api_2::*;
pub use self::app_1::*;

pub const V1: &str = "1.0.0";
pub const V2: &str = "2.0.0";

/// deploys different version apis and app for migration testing
pub fn deploy_modules(mock: &Mock) {
    self::BootMockApi1V1::new(mock.clone())
        .deploy(V1.parse().unwrap(), MockInitMsg)
        .unwrap();

    // do same for version 2
    self::BootMockApi1V2::new(mock.clone())
        .deploy(V2.parse().unwrap(), MockInitMsg)
        .unwrap();

    // and now for api 2
    self::BootMockApi2V1::new(mock.clone())
        .deploy(V1.parse().unwrap(), MockInitMsg)
        .unwrap();

    // do same for version 2
    self::BootMockApi2V2::new(mock.clone())
        .deploy(V2.parse().unwrap(), MockInitMsg)
        .unwrap();

    // and now for app 1
    self::BootMockApp1V1::new(mock.clone())
        .deploy(V1.parse().unwrap())
        .unwrap();

    // do same for version 2
    self::BootMockApp1V2::new(mock.clone())
        .deploy(V2.parse().unwrap())
        .unwrap();
}

pub mod api_1 {
    use super::*;

    pub const MOCK_API_ID: &str = "tester:mock-api1";
    pub use self::v1::*;
    pub use self::v2::*;

    pub mod v1 {

        use super::*;
        gen_api_mock!(BootMockApi1V1, MOCK_API_ID, "1.0.0", &[]);
    }

    pub mod v2 {
        use super::*;
        gen_api_mock!(BootMockApi1V2, MOCK_API_ID, "2.0.0", &[]);
    }
}

pub mod api_2 {
    use super::*;

    pub const MOCK_API_ID: &str = "tester:mock-api2";
    pub use self::v0_1_0::*;
    pub use self::v1::*;
    pub use self::v2_0_0::*;

    pub mod v1 {
        use super::*;
        gen_api_mock!(BootMockApi2V1, MOCK_API_ID, "1.0.0", &[]);
    }

    pub mod v2_0_0 {
        use super::*;
        gen_api_mock!(BootMockApi2V2, MOCK_API_ID, "2.0.0", &[]);
    }

    pub mod v0_1_0 {
        use super::*;
        gen_api_mock!(BootMockApi2V0_1_0, MOCK_API_ID, "0.1.0", &[]);
    }
}

// app 1 depends on api 1 and api 2
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
            "0.1.0",
            &[
                StaticDependency::new(api_1::MOCK_API_ID, &[V1]),
                StaticDependency::new(api_2::MOCK_API_ID, &[V1]),
            ]
        );
    }

    pub mod v2 {
        use super::*;
        gen_app_mock!(
            BootMockApp1V2,
            MOCK_APP_ID,
            "0.2.0",
            &[
                StaticDependency::new(api_1::MOCK_API_ID, &[V2]),
                StaticDependency::new(api_2::MOCK_API_ID, &[V2]),
            ]
        );
    }
}
