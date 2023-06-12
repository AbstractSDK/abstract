use cosmwasm_std::CosmosMsg;

/// Encapsulates an action on the account.
/// When a method returns an AccountAction, this means this message needs to be dispatched to the account using the [`Execution`] api.
///
/// If required you can create an AccountAction from a CosmosMsg using `AccountAction::from(msg)`.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct AccountAction(Vec<CosmosMsg>);

impl AccountAction {
    /// Create a new empty AccountAction
    pub fn new() -> Self {
        Self(vec![])
    }
    /// Access the underlying messages
    /// Don't use this to execute the action, use the `Execution` API instead.
    pub fn messages(&self) -> Vec<CosmosMsg> {
        self.0.clone()
    }
    /// Merge two AccountActions into one.
    pub fn merge(&mut self, other: AccountAction) {
        self.0.extend(other.0)
    }
    /// Creates an account action from multiple messages
    pub fn from_vec<T>(msgs: Vec<T>) -> Self
    where
        T: Into<CosmosMsg>,
    {
        Self(msgs.into_iter().map(Into::into).collect())
    }
}

impl<T> From<T> for AccountAction
where
    T: Into<CosmosMsg>,
{
    fn from(m: T) -> Self {
        Self(vec![m.into()])
    }
}
