//! # Governance structure object

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Governance types
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceDetails {
    /// A single address is admin
    Monarchy {
        /// The monarch's address
        monarch: String,
    },
    /// Fixed multi-sig governance
    MultiSignature {
        /// Number of signatures
        total_members: u8,
        /// Minimum amounts of votes for a proposal to pass
        threshold_votes: u8,
        /// Member addresses, must be of length total_members
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
