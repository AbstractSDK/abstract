use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, StdResult};

use abstract_os::proxy::dapp_base::{BaseExecuteMsg, BaseQueryMsg, BaseStateResponse};

use crate::contract::{execute, query};
use crate::{ExecuteMsg, QueryMsg};
use crate::tests::common::{TEST_CREATOR, TRADER_CONTRACT, TREASURY_CONTRACT};
use crate::tests::instantiate::mock_instantiate;
