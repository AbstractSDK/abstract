use super::{execute::AppExecCtx, query::AppQueryCtx, sdk::SylviaAbstractContract};
use abstract_core::app::{BaseExecuteMsg, BaseInstantiateMsg, BaseMigrateMsg, BaseQueryMsg};

pub struct AbstractApp;

impl SylviaAbstractContract for AbstractApp {
    type BaseInstantiateMsg = BaseInstantiateMsg;
    type BaseMigrateMsg = BaseMigrateMsg;
    type BaseExecuteMsg = BaseExecuteMsg;
    type ExecuteCtx<'a> = AppExecCtx<'a>;
    type BaseQueryMsg = BaseQueryMsg;
    type QueryCtx<'a> = AppQueryCtx<'a>;
}
