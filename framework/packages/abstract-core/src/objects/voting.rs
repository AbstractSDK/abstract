use crate::AbstractResult;
use cosmwasm_std::{ensure, Addr, Decimal, Env, StdError, StdResult, Storage};
use cw_storage_plus::{Bound, Item, Map};
use cw_utils::Duration;

pub const DEFAULT_LIMIT: u64 = 25;
pub type VoteId = u64;

/// Simple voting helper
pub struct SimpleVoting<'a> {
    vote_id: Item<'a, VoteId>,
    votes: Map<'a, (VoteId, &'a Addr), Option<Vote>>,
    vote_config: Item<'a, VoteConfig>,
}

impl<'a> SimpleVoting<'a> {
    pub const fn new(id_key: &'a str, vote_config_key: &'a str, votes_key: &'a str) -> Self {
        Self {
            vote_id: Item::new(id_key),
            votes: Map::new(votes_key),
            vote_config: Item::new(vote_config_key),
        }
    }

    pub fn instantiate(
        &self,
        store: &mut dyn Storage,
        vote_config: &VoteConfig,
    ) -> AbstractResult<()> {
        self.vote_id.save(store, &VoteId::default())?;
        self.vote_config.save(store, vote_config)?;
        Ok(())
    }

    pub fn new_vote(
        &self,
        store: &mut dyn Storage,
        initial_voters: &[Addr],
    ) -> AbstractResult<VoteId> {
        let vote_id = self
            .vote_id
            .update(store, |id| AbstractResult::Ok(id + 1))?
            // to start from zero
            - 1;

        for voter in initial_voters {
            self.votes.save(store, (vote_id, voter), &None);
        }
        Ok(vote_id)
    }

    pub fn cast_vote(
        &self,
        store: &mut dyn Storage,
        vote_id: VoteId,
        voter: &Addr,
        vote: Vote,
        new_voter: bool,
    ) -> AbstractResult<()> {
        match new_voter {
            true => {
                // Need to check it's existing vote id
                let current_vote_id = self.vote_id.load(store)?;
                ensure!(
                    current_vote_id <= vote_id,
                    crate::AbstractError::Std(StdError::generic_err(
                        "There are no vote by this id"
                    ))
                );
                self.votes.save(store, (vote_id, voter), &Some(vote))?;
            }
            false => {
                self.votes.update(
                    store,
                    (vote_id, voter),
                    |previous_vote| match previous_vote {
                        // We allow re-voting
                        Some(_v) => Ok(Some(vote)),
                        None => Err(StdError::generic_err("This user is not allowed to vote")),
                    },
                )?;
            }
        }
        Ok(())
    }

    pub fn count_votes(&self, store: &mut dyn Storage, vote_id: VoteId) -> AbstractResult<()> {
        Ok(())
    }

    pub fn add_voters(
        &self,
        store: &mut dyn Storage,
        vote_id: VoteId,
        new_voters: &[Addr],
    ) -> AbstractResult<()> {
        // Check it's existing vote id
        let current_vote_id = self.vote_id.load(store)?;
        ensure!(
            current_vote_id <= vote_id,
            crate::AbstractError::Std(StdError::generic_err("There are no vote by this id"))
        );

        for voter in new_voters {
            // Don't override already existing vote
            self.votes.update(store, (vote_id, voter), |v| match v {
                Some(v) => AbstractResult::Ok(v),
                None => AbstractResult::Ok(None),
            })?;
        }
        Ok(())
    }

    pub fn remove_voters(
        &self,
        store: &mut dyn Storage,
        vote_id: VoteId,
        removed_voters: &[Addr],
    ) -> AbstractResult<()> {
        for voter in removed_voters {
            self.votes.remove(store, (vote_id, voter));
        }
        Ok(())
    }

    pub fn load_vote(
        &self,
        store: &dyn Storage,
        vote_id: VoteId,
        voter: &Addr,
    ) -> AbstractResult<Option<Vote>> {
        self.votes.load(store, (vote_id, voter)).map_err(Into::into)
    }

    // TODO: do we want pagination on this?
    // I guess no, because if we can't load all votes in one tx means we can't count it
    // so maybe another TODO: Figure out max len for it
    pub fn votes_for_id(
        &self,
        store: &dyn Storage,
        vote_id: VoteId,
    ) -> AbstractResult<Vec<(Addr, Option<Vote>)>> {
        let votes = self
            .votes
            .prefix(vote_id)
            .range(store, None, None, cosmwasm_std::Order::Ascending)
            .collect::<StdResult<_>>()?;
        Ok(votes)
    }

    pub fn query_list(
        &self,
        store: &dyn Storage,
        start_after: Option<(VoteId, &Addr)>,
        limit: Option<u64>,
    ) -> AbstractResult<Vec<((VoteId, Addr), Option<Vote>)>> {
        let min = start_after.map(Bound::exclusive);
        let limit = limit.unwrap_or(DEFAULT_LIMIT);
        let votes = self
            .votes
            .range(store, min, None, cosmwasm_std::Order::Ascending)
            .take(limit as usize)
            .collect::<StdResult<_>>()?;
        Ok(votes)
    }
}

#[cosmwasm_schema::cw_serde]
pub struct Vote {
    pub vote: bool,
    pub memo: Option<String>,
}

#[cosmwasm_schema::cw_serde]
pub struct VoteConfig {
    pub threshold: Threshold,
    pub veto_duration: Duration,
}

#[cosmwasm_schema::cw_serde]
pub enum Threshold {
    Majority {},
    Percentage(Decimal),
}

impl Threshold {
    /// Asserts that the 0.0 < percent <= 1.0
    fn validate_percentage(&self) -> StdResult<()> {
        if let Threshold::Percentage(percent) = self {
            if percent.is_zero() {
                Err(StdError::generic_err("Threshold can't be 0%"))
            } else if *percent > Decimal::one() {
                Err(StdError::generic_err(
                    "Not possible to reach >100% threshold",
                ))
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}
