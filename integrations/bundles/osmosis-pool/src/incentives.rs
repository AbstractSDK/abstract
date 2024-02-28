use osmosis_test_tube::osmosis_std::types::osmosis::incentives::{
    GaugesRequest, GaugesResponse, MsgAddToGauge, MsgAddToGaugeResponse, MsgCreateGauge,
    MsgCreateGaugeResponse, RewardsEstRequest, RewardsEstResponse,
};

use osmosis_test_tube::osmosis_std::types::osmosis::poolincentives::v1beta1::{
    QueryDistrInfoRequest, QueryDistrInfoResponse, QueryExternalIncentiveGaugesRequest,
    QueryExternalIncentiveGaugesResponse, QueryGaugeIdsRequest, QueryGaugeIdsResponse,
    QueryIncentivizedPoolsRequest, QueryLockableDurationsRequest, QueryLockableDurationsResponse,
};
use osmosis_test_tube::{fn_execute, fn_query};
use osmosis_test_tube::{Module, Runner};

// Astroport testtube reference for examples: https://github.com/astroport-fi/astroport-on-osmosis/blob/main/e2e_tests/src/helper.rs
// Boilerplate code, copy and rename should just do the trick
pub struct Incentives<'a, R: Runner<'a>> {
    runner: &'a R,
}

impl<'a, R: Runner<'a>> Module<'a, R> for Incentives<'a, R> {
    fn new(runner: &'a R) -> Self {
        Self { runner }
    }
}
// End Boilerplate code

impl<'a, R> Incentives<'a, R>
where
    R: Runner<'a>,
{
    // macro for creating execute function
    fn_execute! {
        // (pub)? <fn_name>: <request_type> => <response_type>
        pub create_gauge: MsgCreateGauge => MsgCreateGaugeResponse
    }

    fn_execute! {
        pub add_to_gauge: MsgAddToGauge => MsgAddToGaugeResponse
    }

    // macro for creating query function
    fn_query! {
        // (pub)? <fn_name> [<method_path>]: <request_type> => <response_type>
        pub query_gauge_ids ["osmosis.incentives.v1beta1.Query/GaugeIds"]: QueryGaugeIdsRequest => QueryGaugeIdsResponse
    }
    fn_query! {
        pub query_gauges ["osmosis.incentives.v1beta1.Query/Gauges"]: GaugesRequest => GaugesResponse
    }
    fn_query! {
        pub query_rewards_est["osmosis.incentives.v1beta1.Query/RewardsEst"]: RewardsEstRequest => RewardsEstResponse
    }
    fn_query! {
        pub query_distr_info ["osmosis.incentives.v1beta1.Query/DistrInfo"]: QueryDistrInfoRequest => QueryDistrInfoResponse
    }
    fn_query! {
        pub query_incentivised_pools ["osmosis.incentives.v1beta1.Query/IncentivisedPools"]: QueryIncentivizedPoolsRequest => QueryIncentivizedPoolsRequest
    }
    fn_query! {
        pub query_external_incentive_gauges ["osmosis.incentives.v1beta1.Query/ExternalIncentiveGauges"]: QueryExternalIncentiveGaugesRequest => QueryExternalIncentiveGaugesResponse
    }
    fn_query! {
        pub query_lockable_durations ["/osmosis.incentives.Query/LockableDurations"]: QueryLockableDurationsRequest => QueryLockableDurationsResponse
    }
}
