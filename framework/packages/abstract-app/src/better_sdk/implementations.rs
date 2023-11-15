use abstract_core::app::{BaseInstantiateMsg,BaseMigrateMsg, BaseExecuteMsg, BaseQueryMsg};
use super::{sdk::SylviaAbstractContract, execute::AppExecCtx, query::AppQueryCtx};

pub struct AbstractApp;

impl SylviaAbstractContract for AbstractApp{
    type BaseInstantiateMsg = BaseInstantiateMsg;
    type BaseMigrateMsg = BaseMigrateMsg;
    type BaseExecuteMsg = BaseExecuteMsg;
    type ExecuteCtx<'a> = AppExecCtx<'a>;
    type BaseQueryMsg = BaseQueryMsg;
    type QueryCtx<'a> = AppQueryCtx<'a>;
}
