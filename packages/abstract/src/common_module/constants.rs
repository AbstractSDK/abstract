use cw_controllers::Admin;

pub const BASE_STATE_KEY: &str = "\u{0}{10}base_state";
pub const ADMIN_KEY: &str = "admin";
pub const ADMIN: Admin = Admin::new(ADMIN_KEY);
