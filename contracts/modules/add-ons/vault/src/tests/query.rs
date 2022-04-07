use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, StdResult};

use pandora_os::modules::dapp_base::msg::{BaseExecuteMsg, BaseQueryMsg, BaseStateResponse};

use crate::contract::{execute, query};
use pandora_os::modules::add_ons::vault::{ExecuteMsg, QueryMsg};
use crate::tests::common::{TEST_CREATOR, TRADER_CONTRACT, TREASURY_CONTRACT};
use crate::tests::instantiate::mock_instantiate;
