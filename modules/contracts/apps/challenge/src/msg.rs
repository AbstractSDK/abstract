#![warn(missing_docs)]
//! Message types for the challenge app
use crate::{
    contract::ChallengeApp,
    state::{
        AdminStrikes, ChallengeEntry, ChallengeEntryUpdate, StrikeStrategy, UpdateFriendsOpKind,
    },
};
use abstract_core::objects::{
    voting::{ProposalInfo, ProposalStatus, VetoAdminAction, Vote, VoteConfig},
    AccountId, AssetEntry,
};
use abstract_sdk::{AbstractSdkResult, AccountVerification};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Deps, StdResult};
use cw_address_like::AddressLike;
use cw_utils::{Duration, Expiration};

abstract_app::app_msg_types!(ChallengeApp, ChallengeExecuteMsg, ChallengeQueryMsg);

/// Challenge instantiate message
#[cosmwasm_schema::cw_serde]
pub struct ChallengeInstantiateMsg {
    /// Config for [`SimpleVoting`](abstract_core::objects::voting::SimpleVoting) object
    pub vote_config: VoteConfig,
}

/// Challenge execute messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg))]
pub enum ChallengeExecuteMsg {
    /// Create new challenge
    CreateChallenge {
        /// New challenge arguments
        challenge_req: ChallengeRequest,
    },
    /// Update existing challenge
    UpdateChallenge {
        /// Id of the challenge to update
        challenge_id: u64,
        /// Updates to this challenge
        challenge: ChallengeEntryUpdate,
    },
    /// Cancel challenge
    CancelChallenge {
        ///Challenge Id to cancel
        challenge_id: u64,
    },
    /// Update list of friends for challenge
    UpdateFriendsForChallenge {
        /// Id of the challenge to update
        challenge_id: u64,
        /// List of added or removed Friends
        friends: Vec<Friend<String>>,
        /// Kind of operation: add or remove friends
        op_kind: UpdateFriendsOpKind,
    },
    /// Cast vote as a friend
    CastVote {
        /// Challenge Id to cast vote on
        challenge_id: u64,
        /// Wether voter thinks admin deserves punishment
        vote_to_punish: Vote,
    },
    /// Count votes for challenge id
    CountVotes {
        /// Challenge Id for counting votes
        challenge_id: u64,
    },
    /// Finish the Vote that is in veto state
    VetoAction {
        /// Challenge id to do the veto action
        challenge_id: u64,
        /// Veto action for the challenge
        action: VetoChallengeAction,
    },
}

/// Veto actions on challenge
#[cosmwasm_schema::cw_serde]
pub enum VetoChallengeAction {
    /// Admin-only actions
    AdminAction(VetoAdminAction),
    /// Voter-only action, to finish veto state, in case veto period expired
    FinishExpired,
}

/// Challenge query messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum ChallengeQueryMsg {
    /// Get challenge info, will return null if there was no challenge by Id
    #[returns(ChallengeResponse)]
    Challenge {
        /// Id of requested challenge
        challenge_id: u64,
    },
    /// Get list of challenges
    #[returns(ChallengesResponse)]
    Challenges {
        /// start after challenge Id
        start_after: Option<u64>,
        /// Max amount of challenges in response
        limit: Option<u64>,
    },
    /// List of friends by Id
    #[returns(FriendsResponse)]
    Friends {
        /// Id of requested challenge
        challenge_id: u64,
    },
    /// Get vote of friend
    #[returns(VoteResponse)]
    Vote {
        /// Addr of the friend
        voter_addr: String,
        /// Id of requested challenge
        challenge_id: u64,
        /// Index of the previous proposal
        /// Providing None requests current proposal results
        previous_proposal_index: Option<u64>,
    },
    /// Get votes of challenge
    #[returns(VotesResponse)]
    Votes {
        /// Id of requested challenge
        challenge_id: u64,
        /// Index of the previous proposal
        /// Providing None requests current proposal results
        previous_proposal_index: Option<u64>,
        /// start after Addr
        start_after: Option<Addr>,
        /// Max amount of challenges in response
        limit: Option<u64>,
    },
    /// Get results of previous votes for this challenge
    #[returns(PreviousProposalsResponse)]
    PreviousProposals {
        /// Challenge Id for previous votes
        challenge_id: u64,
    },
}
/// Response for previous_vote query
#[cosmwasm_schema::cw_serde]
pub struct VotesResponse {
    /// List of votes by addr
    pub votes: Vec<(Addr, Option<Vote>)>,
}

/// Response for previous_vote query
#[cosmwasm_schema::cw_serde]
pub struct PreviousProposalsResponse {
    /// results of previous proposals
    pub results: Vec<ProposalInfo>,
}

/// Response for challenge query
#[cosmwasm_schema::cw_serde]
pub struct ChallengeResponse {
    /// Challenge info, will return null if there was no challenge by Id
    pub challenge: Option<ChallengeEntryResponse>,
}

/// Response struct for challenge entry
#[cosmwasm_schema::cw_serde]
pub struct ChallengeEntryResponse {
    /// Id of the challenge,
    pub challenge_id: u64,
    /// Name of challenge
    pub name: String,
    /// Asset for punishment for failing a challenge
    pub strike_asset: AssetEntry,
    /// How strike will get distributed between friends
    pub strike_strategy: StrikeStrategy,
    /// Description of the challenge
    pub description: String,
    /// When challenge ends
    pub end: Expiration,
    /// Status of the current vote
    pub status: ProposalStatus,
    /// State of strikes of admin for this challenge
    pub admin_strikes: AdminStrikes,
}

impl ChallengeEntryResponse {
    pub(crate) fn from_entry_and_proposal_info(
        entry: ChallengeEntry,
        challenge_id: u64,
        vote_info: ProposalInfo,
    ) -> Self {
        Self {
            challenge_id,
            name: entry.name,
            strike_asset: entry.strike_asset,
            strike_strategy: entry.strike_strategy,
            description: entry.description,
            end: entry.end,
            status: vote_info.status,
            admin_strikes: entry.admin_strikes,
        }
    }
}

/// Arguments for new challenge
#[cosmwasm_schema::cw_serde]
pub struct ChallengeRequest {
    /// Name of challenge
    pub name: String,
    /// Asset for punishment for failing a challenge
    pub strike_asset: AssetEntry,
    /// How strike will get distributed between friends
    pub strike_strategy: StrikeStrategy,
    /// Description of the challenge
    pub description: String,
    /// In what duration challenge should end
    pub duration: Duration,
    /// Strike limit, defaults to 1
    pub strikes_limit: Option<u8>,
    /// Initial list of friends
    pub init_friends: Vec<Friend<String>>,
}

/// Friend object
#[cosmwasm_schema::cw_serde]
pub enum Friend<T: AddressLike> {
    /// Friend with address and a name
    Addr(FriendByAddr<T>),
    /// Abstract Account Id of the friend
    AbstractAccount(AccountId),
}

impl Friend<String> {
    pub(crate) fn check(
        self,
        deps: Deps,
        app: &ChallengeApp,
    ) -> AbstractSdkResult<(Addr, Friend<Addr>)> {
        let account_registry = app.account_registry(deps);
        let checked = match self {
            Friend::Addr(human) => {
                let checked = human.check(deps)?;
                (checked.address.clone(), Friend::Addr(checked))
            }
            Friend::AbstractAccount(account_id) => {
                let base = account_registry.account_base(&account_id)?;
                (base.manager, Friend::AbstractAccount(account_id))
            }
        };
        Ok(checked)
    }
}

impl Friend<Addr> {
    pub(crate) fn addr(&self, deps: Deps, app: &ChallengeApp) -> AbstractSdkResult<Addr> {
        Ok(match self {
            Friend::Addr(human) => human.address.clone(),
            Friend::AbstractAccount(account_id) => {
                app.account_registry(deps).account_base(account_id)?.manager
            }
        })
    }
}

/// Friend by address
#[cosmwasm_schema::cw_serde]
pub struct FriendByAddr<T: AddressLike> {
    /// Address of the friend
    pub address: T,
    /// Name of the friend
    pub name: String,
}

impl FriendByAddr<String> {
    pub(crate) fn check(self, deps: Deps) -> StdResult<FriendByAddr<Addr>> {
        let checked = deps.api.addr_validate(&self.address)?;
        Ok(FriendByAddr {
            address: checked,
            name: self.name,
        })
    }
}

/// Response for vote query
#[cosmwasm_schema::cw_serde]
pub struct VoteResponse {
    /// The vote, will return null if there was no vote by this user
    pub vote: Option<Vote>,
}

/// Response for challenges query
/// Returns a list of challenges
#[cosmwasm_schema::cw_serde]
pub struct ChallengesResponse {
    /// List of indexed challenges
    pub challenges: Vec<ChallengeEntryResponse>,
}

/// Response for friends query
/// Returns a list of friends
#[cosmwasm_schema::cw_serde]
pub struct FriendsResponse {
    /// List of friends on challenge
    pub friends: Vec<Friend<Addr>>,
}
