pub mod bank;
pub mod execution;
pub mod execution_stack;
pub mod implementations;
pub mod nameservice;
pub mod sdk;
pub mod instantiate;
pub mod execute;
pub mod migrate;
pub mod query;

// TODO: add macro here that generates the private struct below
// The macro should:
// 1. Generate a struct that contains this struct and the ModuleEnv
// 2. Generate a new function that instantiates the struct
// 3. Allow generation of endpoints simply (see sylvia)
