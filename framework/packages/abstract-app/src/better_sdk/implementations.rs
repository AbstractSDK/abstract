use abstract_core::app::{BaseInstantiateMsg,BaseMigrateMsg};
use super::sdk::SylviaAbstractContract;

pub struct AbstractApp;

impl SylviaAbstractContract for AbstractApp{
    type BaseInstantiateMsg = BaseInstantiateMsg;
    type BaseMigrateMsg = BaseMigrateMsg;
}
