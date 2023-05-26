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
}

impl From<CosmosMsg> for AccountAction {
    fn from(m: CosmosMsg) -> Self {
        Self(vec![m])
    }
}

impl From<Vec<CosmosMsg>> for AccountAction {
    fn from(msgs: Vec<CosmosMsg>) -> Self {
        Self(msgs)
    }
}
