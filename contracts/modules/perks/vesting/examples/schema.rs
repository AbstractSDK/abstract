use cosmwasm_schema::write_api;

use abstract_os::vesting::{ExecuteMsg, InstantiateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
    };

    // let mut out_dir = current_dir().unwrap();
    // out_dir.push("schema");
    // create_dir_all(&out_dir).unwrap();
    // remove_schemas(&out_dir).unwrap();

    // export_schema(&schema_for!(InstantiateMsg), &out_dir);
    // export_schema(&schema_for!(ExecuteMsg), &out_dir);
    // export_schema(&schema_for!(ReceiveMsg), &out_dir);
    // export_schema(&schema_for!(QueryMsg), &out_dir);
    // export_schema(&schema_for!(ConfigResponse), &out_dir);
    // export_schema(&schema_for!(StateResponse), &out_dir);
    // export_schema(&schema_for!(SimulateWithdrawResponse), &out_dir);
    // export_schema(&schema_for!(AllocationInfo), &out_dir);
    // export_schema(&schema_for!(Schedule), &out_dir);
}
