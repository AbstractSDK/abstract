# JunoSwap Staking Migration

This contract is intended as a migration target for an existing JunoSwap staking contract.
Upon migration it will prepare for a future transfer (we don't know the exact pool address,
as we want to perform the migration proposal concurrently with the pool deployment proposal).

Once it is migrated, the staking contract will be locked, and the only action is that
a nominated migrator account can trigger the transfer of the LP controlled by the staking pool
to a wynddex pool. We add some constraints to ensure this is transferred to a valid target contract
to minimize any trust requirements on the migrator (they just have "censorship" power).

