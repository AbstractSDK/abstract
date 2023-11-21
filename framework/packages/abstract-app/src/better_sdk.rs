pub mod account_identification;
pub mod bank;
pub mod contexts;
pub mod execution;
pub mod execution_stack;
pub mod mock_module;
pub mod nameservice;
pub mod sdk;

// TODO: add macro here that generates the private struct below
// The macro should:
// 1. Generate a struct that contains this struct and the ModuleEnv
// 2. Generate a new function that instantiates the struct
// 3. Allow generation of endpoints simply (see sylvia)
