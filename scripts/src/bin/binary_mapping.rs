// Test binary mapping inside of CosmWasm contracts

use std::str::FromStr;

use cosmwasm_std::{from_binary, to_binary, Decimal};

use serde::{Deserialize, Serialize};
// Need to use wasm version of serde-value to exclude floating point operations
use serde_cw_value::{self, Value};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TestInit {
    // This admin should be set/overwritten by the factory
    admin: String,
    fee: Decimal,
    description: String,
}

struct OverwriteRule {
    key: String,
    value: Value,
}

pub fn script() -> anyhow::Result<()> {
    let store: Vec<OverwriteRule>;
    let overwrite_admin = OverwriteRule {
        key: "admin".to_string(),
        value: Value::String("abstract_owner".into()),
    };

    let overwrite_fee = OverwriteRule {
        key: "fee".to_string(),
        value: Value::String("0.42".into()),
    };

    store = vec![overwrite_admin, overwrite_fee];

    let input = TestInit {
        admin: "some_other_admin".into(),
        fee: Decimal::from_str("642.62")?,
        description: "my own description".to_string(),
    };

    let bin = to_binary(&input)?;
    let from_bin: Value = from_binary(&bin)?;
    let dubble_check: TestInit = from_binary(&bin)?;

    let mut input_value = serde_cw_value::to_value(from_bin)?;
    println!("{:?}", dubble_check);
    match input_value {
        Value::Map(ref mut val_map) => {
            for rule in store {
                let val = val_map.get_mut(&Value::String(rule.key)).unwrap();
                *val = rule.value;
            }
        }
        _ => panic!(),
    };

    let updated_msg = TestInit::deserialize(input_value).unwrap();
    log::debug!("{:#?}", input);
    log::debug!("{:#?}", updated_msg);

    Ok(())
}

fn main() {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    if let Err(ref err) = script() {
        log::error!("{}", err);
        err.chain()
            .skip(1)
            .for_each(|cause| log::error!("because: {}", cause));

        // The backtrace is not always generated. Try to run this example
        // with `$env:RUST_BACKTRACE=1`.
        //    if let Some(backtrace) = e.backtrace() {
        //        log::debug!("backtrace: {:?}", backtrace);
        //    }

        ::std::process::exit(1);
    }
}
