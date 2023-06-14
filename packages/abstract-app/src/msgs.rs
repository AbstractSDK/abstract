#[macro_export]
/// Groups code that is needed on every app.
/// This registers the types for safety when using Messages
/// This is also used to indicate that The Query And Execute messages or used as app messages
macro_rules! app_messages {
    ($app_type:ty, $app_execute_msg: ty, $app_query_msg: ty) => {
        /// Abstract App instantiate msg
        pub type InstantiateMsg =
            <$app_type as ::abstract_sdk::base::InstantiateEndpoint>::InstantiateMsg;
        pub type ExecuteMsg = <$app_type as ::abstract_sdk::base::ExecuteEndpoint>::ExecuteMsg;
        pub type QueryMsg = <$app_type as ::abstract_sdk::base::QueryEndpoint>::QueryMsg;
        pub type MigrateMsg = <$app_type as ::abstract_sdk::base::MigrateEndpoint>::MigrateMsg;

        impl ::abstract_core::app::AppExecuteMsg for $app_execute_msg {}
        impl ::abstract_core::app::AppQueryMsg for $app_query_msg {}
    };
}
