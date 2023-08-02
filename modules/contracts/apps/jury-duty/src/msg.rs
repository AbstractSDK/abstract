use cosmwasm_std::{Addr, CosmosMsg, Empty};
use cw3::Vote;
use cw4::MemberChangedHookMsg;
use cw_utils::Expiration;

use crate::contract::JuryDutyApp;

abstract_app::app_msg_types!(JuryDutyApp, JuryDutyExecuteMsg, JuryDutyQueryMsg);

pub type JuryDutyInstantiateMsg = cw3_fixed_multisig::msg::InstantiateMsg;

/// App execute messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg))]
pub enum JuryDutyExecuteMsg {
    Propose {
        title: String,
        description: String,
        msgs: Vec<CosmosMsg<Empty>>,
        // note: we ignore API-spec'd earliest if passed, always opens immediately
        latest: Option<Expiration>,
    },
    Vote {
        proposal_id: u64,
        vote: Vote,
    },
    Execute {
        proposal_id: u64,
    },
    Close {
        proposal_id: u64,
    },
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
#[derive(cosmwasm_schema::QueryResponses)]
pub enum JuryDutyQueryMsg {
    // Cw3Query(cw3_fixed_multisig::msg::QueryMsg),
    #[returns(JuryResponse)]
    Jury { proposal_id: u64 },
}

#[cosmwasm_schema::cw_serde]
pub struct JuryResponse {
    pub jury: Option<Vec<Addr>>,
}
