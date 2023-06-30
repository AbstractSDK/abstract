use abstract_ica::{BalancesResponse, DispatchResponse, RegisterResponse, StdAck};
use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use std::{env::current_dir, fs::create_dir_all};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    // export_schema(&schema_for!(PacketMsg), &out_dir);
    export_schema(&schema_for!(StdAck), &out_dir);
    export_schema(&schema_for!(DispatchResponse), &out_dir);
    export_schema(&schema_for!(BalancesResponse), &out_dir);
    export_schema(&schema_for!(RegisterResponse), &out_dir);
}
