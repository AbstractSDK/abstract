#![doc = include_str!("README.md")]

pub use crate::objects::gov_type::{GovAction, GovernanceDetails};
use crate::{objects::common_namespace::OWNERSHIP_STORAGE_KEY, AbstractError};

use cosmwasm_std::{
    Addr, Attribute, BlockInfo, CustomQuery, DepsMut, QuerierWrapper, StdError, StdResult, Storage,
};
use cw_address_like::AddressLike;
use cw_storage_plus::Item;
pub use cw_utils::Expiration;

use super::nested_admin::query_top_level_owner;

/// Errors associated with the contract's ownership
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GovOwnershipError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("Contract ownership has been renounced")]
    NoOwner,

    #[error("Caller is not the contract's current owner")]
    NotOwner,

    #[error("Caller is not the contract's pending owner")]
    NotPendingOwner,

    #[error("There isn't a pending ownership transfer")]
    TransferNotFound,

    #[error("A pending ownership transfer exists but it has expired")]
    TransferExpired,

    #[error("Cannot transfer ownership to renounced structure. use action::renounce")]
    TransferToRenounced,

    #[error("Cannot change NFT ownership. Transfer the NFT to change account ownership.")]
    ChangeOfNftOwned,
}

/// Storage constant for the contract's ownership
const OWNERSHIP: Item<Ownership<Addr>> = Item::new(OWNERSHIP_STORAGE_KEY);

/// The contract's ownership info
#[cosmwasm_schema::cw_serde]
pub struct Ownership<T: AddressLike> {
    /// The contract's current owner.
    pub owner: GovernanceDetails<T>,

    /// The account who has been proposed to take over the ownership.
    /// `None` if there isn't a pending ownership transfer.
    pub pending_owner: Option<GovernanceDetails<T>>,

    /// The deadline for the pending owner to accept the ownership.
    /// `None` if there isn't a pending ownership transfer, or if a transfer
    /// exists and it doesn't have a deadline.
    pub pending_expiry: Option<Expiration>,
}

impl<T: AddressLike> Ownership<T> {
    /// Serializes the current ownership state as attributes which may
    /// be used in a message response. Serialization is done according
    /// to the std::fmt::Display implementation for `T` and
    /// `cosmwasm_std::Expiration` (for `pending_expiry`). If an
    /// ownership field has no value, `"none"` will be serialized.
    ///
    /// Attribute keys used:
    ///  - owner
    ///  - pending_owner
    ///  - pending_expiry
    ///
    /// Callers should take care not to use these keys elsewhere
    /// in their response as CosmWasm will override reused attribute
    /// keys.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cw_utils::Expiration;
    ///
    /// assert_eq!(
    ///     Ownership {
    ///         owner: Some("blue"),
    ///         pending_owner: None,
    ///         pending_expiry: Some(Expiration::Never {})
    ///     }
    ///     .into_attributes(),
    ///     vec![
    ///         Attribute::new("owner", "blue"),
    ///         Attribute::new("pending_owner", "none"),
    ///         Attribute::new("pending_expiry", "expiration: never")
    ///     ],
    /// )
    /// ```
    pub fn into_attributes(self) -> Vec<Attribute> {
        fn none_or<T: std::fmt::Display>(or: Option<&T>) -> String {
            or.map_or_else(|| "none".to_string(), |or| or.to_string())
        }
        vec![
            Attribute::new("owner", self.owner.to_string()),
            Attribute::new("pending_owner", none_or(self.pending_owner.as_ref())),
            Attribute::new("pending_expiry", none_or(self.pending_expiry.as_ref())),
        ]
    }

    /// Assert current owner supports governance change
    pub fn assert_can_change_owner(&self) -> Result<(), GovOwnershipError> {
        if let GovernanceDetails::NFT { .. } = self.owner {
            return Err(GovOwnershipError::ChangeOfNftOwned);
        }

        Ok(())
    }
}

impl Ownership<Addr> {
    /// Assert that an account is the contract's current owner.
    fn check_owner(
        &self,
        querier: &QuerierWrapper,
        sender: &Addr,
    ) -> Result<(), GovOwnershipError> {
        // the contract must have an owner
        let Some(current_owner) = &self.owner.owner_address(querier) else {
            return Err(GovOwnershipError::NoOwner);
        };

        // the sender must be the current owner
        if sender != current_owner {
            return Err(GovOwnershipError::NotOwner);
        }

        Ok(())
    }

    /// Asserts governance change allowed and account is the contract's current owner.
    fn check_owner_can_change(
        &self,
        querier: &QuerierWrapper,
        sender: &Addr,
    ) -> Result<(), GovOwnershipError> {
        match &self.owner {
            GovernanceDetails::SubAccount { manager, .. } => {
                let top_level_owner = query_top_level_owner(querier, manager.clone())?;
                // Verify top level account allows ownership changes
                // We prevent transfers of current ownership if it's NFT
                top_level_owner.assert_can_change_owner()?;

                // Assert admin
                // We are dealing with sub account, so we need to check both manager as caller and top level address
                if self.check_owner(querier, sender).is_err() {
                    top_level_owner.check_owner(querier, sender)?
                }
            }
            _ => {
                // Verify account allows ownership changes
                // We prevent transfers of current ownership if it's NFT
                self.assert_can_change_owner()?;

                // Assert admin
                self.check_owner(querier, sender)?;
            }
        }

        Ok(())
    }
}

/// Set the given address as the contract owner.
///
/// This function is only intended to be used only during contract instantiation.
pub fn initialize_owner(
    deps: DepsMut,
    owner: GovernanceDetails<String>,
    version_control: Addr,
) -> Result<Ownership<Addr>, GovOwnershipError> {
    let ownership = Ownership {
        owner: owner.verify(deps.as_ref(), version_control)?,
        pending_owner: None,
        pending_expiry: None,
    };
    OWNERSHIP.save(deps.storage, &ownership)?;
    Ok(ownership)
}

/// Return Ok(true) if the contract has an owner and it's the given address.
/// Return Ok(false) if the contract doesn't have an owner, of if it does but
/// it's not the given address.
/// Return Err if fails to load ownership info from storage.
pub fn is_owner(store: &dyn Storage, querier: &QuerierWrapper, addr: &Addr) -> StdResult<bool> {
    let ownership = OWNERSHIP.load(store)?;

    if let Some(owner) = ownership.owner.owner_address(querier) {
        if *addr == owner {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Assert that an account is the contract's current owner.
pub fn assert_owner(
    store: &dyn Storage,
    querier: &QuerierWrapper,
    sender: &Addr,
) -> Result<(), GovOwnershipError> {
    let ownership = OWNERSHIP.load(store)?;
    // If current sender is owner of this account - it's the owner
    if ownership.check_owner(querier, sender).is_ok() {
        return Ok(());
    }
    // Otherwise we need to check top level owner
    let top_level_ownership = if let GovernanceDetails::SubAccount { manager, .. } = ownership.owner
    {
        query_top_level_owner(querier, manager)?
    } else {
        ownership
    };
    // the contract must have an owner
    top_level_ownership.check_owner(querier, sender)
}

/// Update the contract's ownership info based on the given action.
/// Return the updated ownership.
pub fn update_ownership(
    deps: DepsMut,
    block: &BlockInfo,
    sender: &Addr,
    version_control: Addr,
    action: GovAction,
) -> Result<Ownership<Addr>, GovOwnershipError> {
    match action {
        GovAction::TransferOwnership { new_owner, expiry } => {
            transfer_ownership(deps, sender, new_owner, version_control, expiry)
        }
        GovAction::AcceptOwnership => accept_ownership(deps.storage, &deps.querier, block, sender),
        GovAction::RenounceOwnership => renounce_ownership(deps.storage, &deps.querier, sender),
    }
}

/// Get the current ownership value.
pub fn get_ownership(storage: &dyn Storage) -> StdResult<Ownership<Addr>> {
    OWNERSHIP.load(storage)
}

pub fn query_ownership<Q: CustomQuery>(
    querier: &QuerierWrapper<Q>,
    remote_contract: Addr,
) -> StdResult<Ownership<Addr>> {
    OWNERSHIP.query(querier, remote_contract)
}

/// Propose to transfer the contract's ownership to the given address, with an
/// optional deadline.
fn transfer_ownership(
    deps: DepsMut,
    sender: &Addr,
    new_owner: GovernanceDetails<String>,
    version_control: Addr,
    expiry: Option<Expiration>,
) -> Result<Ownership<Addr>, GovOwnershipError> {
    let new_owner = new_owner.verify(deps.as_ref(), version_control)?;

    if new_owner.owner_address(&deps.querier).is_none() {
        return Err(GovOwnershipError::TransferToRenounced {});
    }

    OWNERSHIP.update(deps.storage, |ownership| {
        // Check sender and verify governance is not immutable
        ownership.check_owner_can_change(&deps.querier, sender)?;
        // NOTE: We don't validate the expiry, i.e. asserting it is later than
        // the current block time.
        //
        // This is because if the owner submits an invalid expiry, it won't have
        // any negative effect - it's just that the pending owner won't be able
        // to accept the ownership.
        //
        // By not doing the check, we save a little bit of gas.
        //
        // To fix the error, the owner can simply invoke `transfer_ownership`
        // again with the correct expiry and overwrite the invalid one.
        Ok(Ownership {
            pending_owner: Some(new_owner),
            pending_expiry: expiry,
            ..ownership
        })
    })
}

/// Accept a pending ownership transfer.
fn accept_ownership(
    store: &mut dyn Storage,
    querier: &QuerierWrapper,
    block: &BlockInfo,
    sender: &Addr,
) -> Result<Ownership<Addr>, GovOwnershipError> {
    OWNERSHIP.update(store, |ownership| {
        // there must be an existing ownership transfer
        let Some(maybe_pending_owner) = ownership.pending_owner else {
            return Err(GovOwnershipError::TransferNotFound);
        };

        // If new gov has no owner they cannot accept
        let Some(pending_owner) = maybe_pending_owner.owner_address(querier) else {
            // It's most likely burned NFT or corrupted NFT contract after proposal
            // Make sure to not "renounce" ownership accidentally.
            //
            // P.S. GovAction::RenounceOwnership still available to the original owner if that was intentional
            return Err(GovOwnershipError::TransferNotFound);
        };

        let is_pending_owner = if sender == pending_owner {
            true
        } else if let GovernanceDetails::SubAccount { manager, .. } = &maybe_pending_owner {
            // If not direct owner, need to check top level ownership

            // Check if top level owner of pending is caller
            query_top_level_owner(querier, manager.clone())?
                .owner
                .owner_address(querier)
                .map(|top_sender| top_sender == sender)
                .unwrap_or_default()
        } else {
            false
        };

        // The sender must be the pending owner
        if !is_pending_owner {
            return Err(GovOwnershipError::NotPendingOwner);
        }

        // if the transfer has a deadline, it must not have been reached
        if let Some(expiry) = &ownership.pending_expiry {
            if expiry.is_expired(block) {
                return Err(GovOwnershipError::TransferExpired);
            }
        }

        Ok(Ownership {
            owner: maybe_pending_owner,
            pending_owner: None,
            pending_expiry: None,
        })
    })
}

/// Set the contract's ownership as vacant permanently.
fn renounce_ownership(
    store: &mut dyn Storage,
    querier: &QuerierWrapper,
    sender: &Addr,
) -> Result<Ownership<Addr>, GovOwnershipError> {
    OWNERSHIP.update(store, |ownership| {
        // Check sender and verify governance is not immutable
        ownership.check_owner_can_change(querier, sender)?;

        Ok(Ownership {
            owner: GovernanceDetails::Renounced {},
            pending_owner: None,
            pending_expiry: None,
        })
    })
}

//------------------------------------------------------------------------------
// Tests
//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use cosmwasm_std::{testing::mock_dependencies, Attribute, Timestamp};

    use super::*;

    fn mock_govs() -> [GovernanceDetails<Addr>; 3] {
        [
            GovernanceDetails::Monarchy {
                monarch: Addr::unchecked("larry"),
            },
            GovernanceDetails::Monarchy {
                monarch: Addr::unchecked("jake"),
            },
            GovernanceDetails::Monarchy {
                monarch: Addr::unchecked("pumpkin"),
            },
        ]
    }

    fn vc_addr() -> Addr {
        Addr::unchecked("version_control")
    }

    fn mock_block_at_height(height: u64) -> BlockInfo {
        BlockInfo {
            height,
            time: Timestamp::from_seconds(10000),
            chain_id: "".into(),
        }
    }

    #[test]
    fn initializing_ownership() {
        let mut deps = mock_dependencies();
        let [larry, _, _] = mock_govs();

        let ownership = initialize_owner(deps.as_mut(), larry.clone().into(), vc_addr()).unwrap();

        // ownership returned is same as ownership stored.
        assert_eq!(ownership, OWNERSHIP.load(deps.as_ref().storage).unwrap());

        assert_eq!(
            ownership,
            Ownership {
                owner: larry,
                pending_owner: None,
                pending_expiry: None,
            },
        );
    }

    #[test]
    fn initialize_ownership_no_owner() {
        let mut deps = mock_dependencies();
        let ownership =
            initialize_owner(deps.as_mut(), GovernanceDetails::Renounced {}, vc_addr()).unwrap();
        assert_eq!(
            ownership,
            Ownership {
                owner: GovernanceDetails::Renounced {},
                pending_owner: None,
                pending_expiry: None,
            },
        );
    }

    #[test]
    fn asserting_ownership() {
        let mut deps = mock_dependencies();
        let [larry, jake, _] = mock_govs();
        let larry_address = larry.owner_address(&deps.as_ref().querier).unwrap();
        let jake_address = jake.owner_address(&deps.as_ref().querier).unwrap();

        // case 1. owner has not renounced
        {
            initialize_owner(deps.as_mut(), larry.clone().into(), vc_addr()).unwrap();

            let res = assert_owner(
                deps.as_ref().storage,
                &deps.as_ref().querier,
                &larry_address,
            );
            assert!(res.is_ok());

            let res = assert_owner(deps.as_ref().storage, &deps.as_ref().querier, &jake_address);
            assert_eq!(res.unwrap_err(), GovOwnershipError::NotOwner);
        }

        // case 2. owner has renounced
        {
            let depsmut = deps.as_mut();
            renounce_ownership(depsmut.storage, &depsmut.querier, &larry_address).unwrap();

            let res = assert_owner(
                deps.as_ref().storage,
                &deps.as_ref().querier,
                &larry_address,
            );
            assert_eq!(res.unwrap_err(), GovOwnershipError::NoOwner);
        }
    }

    #[test]
    fn transferring_ownership() {
        let mut deps = mock_dependencies();
        let [larry, jake, pumpkin] = mock_govs();
        let larry_address = larry.owner_address(&deps.as_ref().querier).unwrap();
        let jake_address = jake.owner_address(&deps.as_ref().querier).unwrap();

        initialize_owner(deps.as_mut(), larry.clone().into(), vc_addr()).unwrap();

        // non-owner cannot transfer ownership
        {
            let depsmut = deps.as_mut();

            let err = update_ownership(
                depsmut,
                &mock_block_at_height(12345),
                &jake_address,
                vc_addr(),
                GovAction::TransferOwnership {
                    new_owner: pumpkin.clone().into(),
                    expiry: None,
                },
            )
            .unwrap_err();
            assert_eq!(err, GovOwnershipError::NotOwner);
        }

        // owner properly transfers ownership
        {
            let ownership = update_ownership(
                deps.as_mut(),
                &mock_block_at_height(12345),
                &larry_address,
                vc_addr(),
                GovAction::TransferOwnership {
                    new_owner: pumpkin.clone().into(),
                    expiry: Some(Expiration::AtHeight(42069)),
                },
            )
            .unwrap();
            assert_eq!(
                ownership,
                Ownership {
                    owner: larry,
                    pending_owner: Some(pumpkin),
                    pending_expiry: Some(Expiration::AtHeight(42069)),
                },
            );

            let saved_ownership = OWNERSHIP.load(deps.as_ref().storage).unwrap();
            assert_eq!(saved_ownership, ownership);
        }
    }

    #[test]
    fn accepting_ownership() {
        let mut deps = mock_dependencies();
        let [larry, jake, pumpkin] = mock_govs();
        let larry_address = larry.owner_address(&deps.as_ref().querier).unwrap();
        let jake_address = jake.owner_address(&deps.as_ref().querier).unwrap();
        let pumpkin_address = pumpkin.owner_address(&deps.as_ref().querier).unwrap();

        initialize_owner(deps.as_mut(), larry.clone().into(), vc_addr()).unwrap();

        // cannot accept ownership when there isn't a pending ownership transfer
        {
            let err = update_ownership(
                deps.as_mut(),
                &mock_block_at_height(12345),
                &pumpkin_address,
                vc_addr(),
                GovAction::AcceptOwnership,
            )
            .unwrap_err();
            assert_eq!(err, GovOwnershipError::TransferNotFound);
        }

        transfer_ownership(
            deps.as_mut(),
            &larry_address,
            pumpkin.clone().into(),
            vc_addr(),
            Some(Expiration::AtHeight(42069)),
        )
        .unwrap();

        // non-pending owner cannot accept ownership
        {
            let err = update_ownership(
                deps.as_mut(),
                &mock_block_at_height(12345),
                &jake_address,
                vc_addr(),
                GovAction::AcceptOwnership,
            )
            .unwrap_err();
            assert_eq!(err, GovOwnershipError::NotPendingOwner);
        }

        // cannot accept ownership if deadline has passed
        {
            let err = update_ownership(
                deps.as_mut(),
                &mock_block_at_height(69420),
                &pumpkin_address,
                vc_addr(),
                GovAction::AcceptOwnership,
            )
            .unwrap_err();
            assert_eq!(err, GovOwnershipError::TransferExpired);
        }

        // pending owner properly accepts ownership before deadline
        {
            let ownership = update_ownership(
                deps.as_mut(),
                &mock_block_at_height(10000),
                &pumpkin_address,
                vc_addr(),
                GovAction::AcceptOwnership,
            )
            .unwrap();
            assert_eq!(
                ownership,
                Ownership {
                    owner: pumpkin,
                    pending_owner: None,
                    pending_expiry: None,
                },
            );

            let saved_ownership = OWNERSHIP.load(deps.as_ref().storage).unwrap();
            assert_eq!(saved_ownership, ownership);
        }
    }

    #[test]
    fn renouncing_ownership() {
        let mut deps = mock_dependencies();
        let [larry, jake, pumpkin] = mock_govs();
        let larry_address = larry.owner_address(&deps.as_ref().querier).unwrap();
        let jake_address = jake.owner_address(&deps.as_ref().querier).unwrap();

        let ownership = Ownership {
            owner: larry.clone(),
            pending_owner: Some(pumpkin),
            pending_expiry: None,
        };
        OWNERSHIP.save(deps.as_mut().storage, &ownership).unwrap();

        // non-owner cannot renounce
        {
            let err = update_ownership(
                deps.as_mut(),
                &mock_block_at_height(12345),
                &jake_address,
                vc_addr(),
                GovAction::RenounceOwnership,
            )
            .unwrap_err();
            assert_eq!(err, GovOwnershipError::NotOwner);
        }

        // owner properly renounces
        {
            let ownership = update_ownership(
                deps.as_mut(),
                &mock_block_at_height(12345),
                &larry_address,
                vc_addr(),
                GovAction::RenounceOwnership,
            )
            .unwrap();

            // ownership returned is same as ownership stored.
            assert_eq!(ownership, OWNERSHIP.load(deps.as_ref().storage).unwrap());

            assert_eq!(
                ownership,
                Ownership {
                    owner: GovernanceDetails::Renounced {},
                    pending_owner: None,
                    pending_expiry: None,
                },
            );
        }

        // cannot renounce twice
        {
            let err = update_ownership(
                deps.as_mut(),
                &mock_block_at_height(12345),
                &larry_address,
                vc_addr(),
                GovAction::RenounceOwnership,
            )
            .unwrap_err();
            assert_eq!(err, GovOwnershipError::NoOwner);
        }
    }

    #[test]
    fn into_attributes_works() {
        use cw_utils::Expiration;
        assert_eq!(
            Ownership {
                owner: GovernanceDetails::Monarchy {
                    monarch: "batman".to_string()
                },
                pending_owner: None,
                pending_expiry: Some(Expiration::Never {})
            }
            .into_attributes(),
            vec![
                Attribute::new("owner", "monarch"),
                Attribute::new("pending_owner", "none"),
                Attribute::new("pending_expiry", "expiration: never")
            ],
        );
    }
}
