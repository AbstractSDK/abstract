mod apis;
mod apps;
mod perks;

/// ID of the module
pub type ModuleId<'a> = &'a str;

pub use apis::*;
pub use apps::*;
pub use perks::*;
