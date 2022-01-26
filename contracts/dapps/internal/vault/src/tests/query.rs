use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, StdResult};

use pandora::treasury::dapp_base::msg::{BaseExecuteMsg, BaseQueryMsg, BaseStateResponse};

use crate::contract::{execute, query};
use crate::msg::{ExecuteMsg, QueryMsg};
use crate::tests::common::{TEST_CREATOR, TRADER_CONTRACT, TREASURY_CONTRACT};
use crate::tests::instantiate::mock_instantiate;
