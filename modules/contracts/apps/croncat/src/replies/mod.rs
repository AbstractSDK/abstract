mod execute;

pub use execute::{create_task_reply, task_remove_reply};

pub const TASK_CREATE_REPLY_ID: u64 = 1u64;

pub const TASK_REMOVE_REPLY_ID: u64 = 2u64;
