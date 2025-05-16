/// Integration is used to test the adapter implementation
pub mod integration;
/// Live is used to test the adapter deployment and make sure it's working as expected on the live chain
pub mod live;

// Use https://hermes.pyth.network/docs/#/rest/latest_price_updates to query latest update
pub const ORACLE_PRICE_API: &str = "https://hermes.pyth.network/v2/updates/price/latest?ids%5B%5D=";
pub const PRICE_SOURCE_KEY: &str =
    "e62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43";

pub mod pyth_api {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct PythApiResponse {
        pub binary: PythApiResponseBinary,
        pub parsed: Vec<PythApiResponseparsed>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct PythApiResponseBinary {
        pub data: Vec<String>,
    }
    #[derive(Serialize, Deserialize)]
    pub struct PythApiResponseparsed {
        pub price: PythApiResponsePrice,
    }
    #[derive(Serialize, Deserialize)]
    pub struct PythApiResponsePrice {
        pub price: String,
    }
}
