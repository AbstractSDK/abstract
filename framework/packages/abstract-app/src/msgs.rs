#[macro_export]
/// Generates boilerplate code and entrypoint message types.
///
/// ### Usage
///
/// Requires three arguments:
/// 1. The App type.
/// 2. The App's custom execute message type.
/// 3. The App's custom query message type.
///
/// ```rust,ignore
/// abstract_app::app_msg_types!(MyApp, MyAppExecuteMsg, MyAppQueryMsg);
/// ```
///
/// Generates:
/// ```ignore
/// // These are the entry point messages expected by the smart-contract. Our custom messages get wrapped by the abstract base message.
/// pub type InstantiateMsg =
///     <MyApp as sdk::base::InstantiateEndpoint>::InstantiateMsg;
/// pub type ExecuteMsg = <MyApp as sdk::base::ExecuteEndpoint>::ExecuteMsg;
/// pub type QueryMsg = <MyApp as sdk::base::QueryEndpoint>::QueryMsg;
/// pub type MigrateMsg = <MyApp as sdk::base::MigrateEndpoint>::MigrateMsg;

/// // Implements the trait-bounds for the abstract app messages, which allows them to be used in the App type.
/// // Also implements `Into<ExecuteMsg> for MyAppExecuteMsg` and `Into<QueryMsg> for MyAppQueryMsg`.
/// // This enables the use of the `impl_into` macro of cw-orchestrator.
/// impl abstract_std::app::AppExecuteMsg for MyAppExecuteMsg {}
/// impl abstract_std::app::AppQueryMsg for MyAppQueryMsg {}
/// ```
macro_rules! app_msg_types {
    ($app_type:ty, $app_execute_msg: ty, $app_query_msg: ty) => {
        /// Top-level Abstract App instantiate message. This is the message that is passed to the `instantiate` entrypoint of the smart-contract.
        pub type InstantiateMsg =
            <$app_type as $crate::sdk::base::InstantiateEndpoint>::InstantiateMsg;
        /// Top-level Abstract App execute message. This is the message that is passed to the `execute` entrypoint of the smart-contract.
        pub type ExecuteMsg = <$app_type as $crate::sdk::base::ExecuteEndpoint>::ExecuteMsg;
        /// Top-level Abstract App query message. This is the message that is passed to the `query` entrypoint of the smart-contract.
        pub type QueryMsg = <$app_type as $crate::sdk::base::QueryEndpoint>::QueryMsg;
        /// Top-level Abstract App migrate message. This is the message that is passed to the `migrate` entrypoint of the smart-contract.
        pub type MigrateMsg = <$app_type as $crate::sdk::base::MigrateEndpoint>::MigrateMsg;

        impl $crate::std::app::AppExecuteMsg for $app_execute_msg {}
        impl $crate::std::app::AppQueryMsg for $app_query_msg {}
    };
}
