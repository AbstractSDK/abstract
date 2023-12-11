use cosmwasm_std::{CosmosMsg, ReplyOn};

use crate::{AbstractSdkError, AbstractSdkResult};

#[derive(Debug, PartialEq, Clone)]
pub struct ReplyOptions {
    pub reply_on: ReplyOn,
    pub id: u64,
    pub with_data: bool,
}
#[derive(Default, PartialEq, Clone, Debug)]
pub struct ExecuteOptions {
    pub reply: Option<ReplyOptions>,
}
/// Encapsulates an action on the account.
/// When a method returns an AccountAction, this means this message needs to be dispatched to the account using the [`Execution`](crate::Execution) api.
///
/// If required you can create an AccountAction from a CosmosMsg using `AccountAction::from(msg)`.
#[derive(Debug, PartialEq, Clone, Default)]
#[must_use = "Pass AccountAction to the Executor, see the docs"]
pub struct AccountAction {
    msgs: Vec<CosmosMsg>,
    execute_options: ExecuteOptions,
}

impl AccountAction {
    /// Create a new empty AccountAction
    pub fn new() -> Self {
        Self::default()
    }
    /// Access the underlying messages
    pub fn messages(&self) -> Vec<CosmosMsg> {
        self.msgs.clone()
    }
    /// Access the underlying messages
    pub fn options(&self) -> ExecuteOptions {
        self.execute_options.clone()
    }
    /// Creates an account action from multiple messages
    pub fn from_vec<T>(msgs: Vec<T>) -> Self
    where
        T: Into<CosmosMsg>,
    {
        Self {
            msgs: msgs.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
    /// Creates an account action from multiple messages
    pub fn from_vec_with_options<T>(
        msgs: Vec<T>,
        options: ExecuteOptions,
    ) -> AbstractSdkResult<Self>
    where
        T: Into<CosmosMsg>,
    {
        let mut action = Self::default();

        // We validate the options
        if let Some(reply_options) = action.execute_options.reply {
            if reply_options.with_data && action.msgs.len() != 1 {
                return Err(AbstractSdkError::TooMuchMessages {
                    msgs: msgs.into_iter().map(Into::into).collect(),
                });
            }
        }

        action.msgs = msgs.into_iter().map(Into::into).collect();
        action.execute_options = options;
        Ok(action)
    }
}

impl<T> From<T> for AccountAction
where
    T: Into<CosmosMsg>,
{
    fn from(m: T) -> Self {
        Self::from_vec(vec![m])
    }
}
