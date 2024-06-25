use abstract_std::objects::{namespace::Namespace, AccountId};
use cosmwasm_std::Addr;

/// Represents the a route to fetch an account from.
pub enum AccountSource {
    /// Get the account from a registered [`Namespace`].
    Namespace(Namespace),
    /// Get the account from an [`AccountId`].
    AccountId(AccountId),
    /// Get the account from the address of an installed App.
    App(Addr),
}

impl From<Namespace> for AccountSource {
    fn from(namespace: Namespace) -> Self {
        AccountSource::Namespace(namespace)
    }
}

impl From<AccountId> for AccountSource {
    fn from(account_id: AccountId) -> Self {
        AccountSource::AccountId(account_id)
    }
}
