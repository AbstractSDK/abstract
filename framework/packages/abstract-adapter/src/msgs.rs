#[macro_export]
/// Generates boilerplate code and entrypoint message types.
///
/// ### Usage
///
/// Requires three arguments:
/// 1. The Adapter type.
/// 2. The Adapter's custom execute message type.
/// 3. The Adapter's custom query message type.
///
/// ```rust,ignore
/// abstract_adapter::adapter_msg_types!(MyAdapter, MyAdapterExecuteMsg, MyAdapterQueryMsg);
/// ```
///
/// Generates:
/// ```ignore
/// // These are the entry point messages expected by the smart-contract. Our custom messages get wrapped by the abstract base message.
/// pub type InstantiateMsg =
///     <MyAdapter as abstract_sdk::base::InstantiateEndpoint>::InstantiateMsg;
/// pub type ExecuteMsg = <MyAdapter as abstract_sdk::base::ExecuteEndpoint>::ExecuteMsg;
/// pub type QueryMsg = <MyAdapter as abstract_sdk::base::QueryEndpoint>::QueryMsg;

/// // Implements the trait-bounds for the abstract adapter messages, which allows them to be used in the Adapter type.
/// // Also implements `Into<ExecuteMsg> for MyAdapterExecuteMsg` and `Into<QueryMsg> for MyAdapterQueryMsg`.
/// // This enables the use of the `impl_into` macro of cw-orchestrator.
/// impl abstract_std::adapter::AdapterExecuteMsg for MyAdapterExecuteMsg {}
/// impl abstract_std::adapter::AdapterQueryMsg for MyAdapterQueryMsg {}
/// ```
macro_rules! adapter_msg_types {
    ($adapter_type:ty, $adapter_execute_msg: ty, $adapter_query_msg: ty) => {
        /// Top-level Abstract Adapter instantiate message. This is the message that is passed to the `instantiate` entrypoint of the smart-contract.
        pub type InstantiateMsg =
            <$adapter_type as ::abstract_sdk::base::InstantiateEndpoint>::InstantiateMsg;
        /// Top-level Abstract Adapter execute message. This is the message that is passed to the `execute` entrypoint of the smart-contract.
        pub type ExecuteMsg = <$adapter_type as ::abstract_sdk::base::ExecuteEndpoint>::ExecuteMsg;
        /// Top-level Abstract Adapter query message. This is the message that is passed to the `query` entrypoint of the smart-contract.
        pub type QueryMsg = <$adapter_type as ::abstract_sdk::base::QueryEndpoint>::QueryMsg;

        impl ::abstract_std::adapter::AdapterExecuteMsg for $adapter_execute_msg {}
        impl ::abstract_std::adapter::AdapterQueryMsg for $adapter_query_msg {}
    };
}
