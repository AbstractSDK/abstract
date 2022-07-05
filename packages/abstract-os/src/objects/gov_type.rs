use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceDetails {
    Monarchy {
        monarch: String,
    },
    MultiSignature {
        total_members: u8,
        threshold_votes: u8,
        members: Vec<String>,
    },
    /// An external governance source
    External {
        /// The external contract address
        governance_address: String,
        /// Governance type used for doing extra off-chain queries depending on the type.
        governance_type: String,
    },
}

impl ToString for GovernanceDetails {
    fn to_string(&self) -> String {
        match self {
            GovernanceDetails::Monarchy { monarch: _ } => "monarchy".to_string(),
            GovernanceDetails::MultiSignature {
                total_members: _,
                threshold_votes: _,
                members: _,
            } => "multisig".to_string(),
            GovernanceDetails::External {
                governance_address: _,
                governance_type,
            } => governance_type.clone(),
        }
    }
}
