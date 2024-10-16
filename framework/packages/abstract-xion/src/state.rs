pub const AUTHENTICATORS: cw_storage_plus::Map<u8, crate::auth::Authenticator> =
    cw_storage_plus::Map::new("authenticators");
pub const AUTH_ADMIN: cw_storage_plus::Item<bool> =
    cw_storage_plus::Item::new(abstract_std::objects::storage_namespaces::account::AUTH_ADMIN);
