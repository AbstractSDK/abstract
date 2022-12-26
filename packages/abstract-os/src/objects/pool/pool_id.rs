use cosmwasm_std::{Addr, Api, StdError, StdResult};

use std::fmt;

use std::str::FromStr;

#[cosmwasm_schema::cw_serde]
#[non_exhaustive]
pub enum PoolIdBase<T> {
    Contract(T),
    Id(u64),
}

impl<T> PoolIdBase<T> {
    pub fn contract<C: Into<T>>(contract: C) -> Self {
        Self::Contract(contract.into())
    }
    pub fn id<N: Into<u64>>(id: N) -> Self {
        Self::Id(id.into())
    }
}

/// Actual instance of a PoolId with verified data
pub type PoolId = PoolIdBase<Addr>;

impl PoolId {
    pub fn expect_contract(&self) -> StdResult<Addr> {
        match self {
            PoolId::Contract(addr) => Ok(addr.clone()),
            _ => Err(StdError::generic_err("Not a contract address pool ID.")),
        }
    }

    pub fn expect_id(&self) -> StdResult<u64> {
        match self {
            PoolId::Id(id) => Ok(*id),
            _ => Err(StdError::generic_err("Not an numerical pool ID.")),
        }
    }
}
/// Instance of a PoolId passed around messages
pub type UncheckedPoolId = PoolIdBase<String>;

impl FromStr for UncheckedPoolId {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let words: Vec<&str> = s.split(':').collect();

        match words[0] {
            "contract" => {
                if words.len() != 2 {
                    return Err(StdError::generic_err(
                        format!("invalid pool id format `{}`; must be in format `contract:{{contract_addr}}`", s)
                    ));
                }
                Ok(UncheckedPoolId::Contract(String::from(words[1])))
            }
            "id" => {
                if words.len() != 2 {
                    return Err(StdError::generic_err(format!(
                        "invalid pool id format `{}`; must be in format `id:{{pool_id}}`",
                        s
                    )));
                }
                let parsed_id_res = words[1].parse::<u64>();
                match parsed_id_res {
                    Ok(id) => Ok(UncheckedPoolId::Id(id)),
                    Err(err) => Err(StdError::generic_err(err.to_string())),
                }
            }
            unknown => Err(StdError::generic_err(format!(
                "invalid pool id type `{}`; must be `contract` or `id`",
                unknown
            ))),
        }
    }
}

impl From<PoolId> for UncheckedPoolId {
    fn from(pool_info: PoolId) -> Self {
        match pool_info {
            PoolId::Contract(contract_addr) => UncheckedPoolId::Contract(contract_addr.into()),
            PoolId::Id(denom) => UncheckedPoolId::Id(denom),
        }
    }
}

impl From<&PoolId> for UncheckedPoolId {
    fn from(pool_id: &PoolId) -> Self {
        match pool_id {
            PoolId::Contract(contract_addr) => UncheckedPoolId::Contract(contract_addr.into()),
            PoolId::Id(denom) => UncheckedPoolId::Id(*denom),
        }
    }
}

impl From<Addr> for PoolId {
    fn from(contract_addr: Addr) -> Self {
        PoolId::Contract(contract_addr)
    }
}

impl UncheckedPoolId {
    /// Validate data contained in an _unchecked_ **pool id** instance; return a new _checked_
    /// **pool id** instance:
    /// * For Contract addresses, assert its address is valid
    ///
    ///
    /// ```rust
    /// use cosmwasm_std::{Addr, Api, StdResult};
    /// use abstract_os::objects::pool_id::UncheckedPoolId;
    ///
    /// fn validate_pool_id(api: &dyn Api, pool_id_unchecked: &UncheckedPoolId) {
    ///     match pool_id_unchecked.check(api) {
    ///         Ok(info) => println!("pool id is valid: {}", info.to_string()),
    ///         Err(err) => println!("pool id is invalid! reason: {}", err),
    ///     }
    /// }
    /// ```
    pub fn check(&self, api: &dyn Api) -> StdResult<PoolId> {
        Ok(match self {
            UncheckedPoolId::Contract(contract_addr) => {
                PoolId::Contract(api.addr_validate(contract_addr)?)
            }
            UncheckedPoolId::Id(pool_id) => PoolId::Id(*pool_id),
        })
    }
}

impl fmt::Display for PoolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PoolId::Contract(contract_addr) => write!(f, "contract:{}", contract_addr),
            PoolId::Id(pool_id) => write!(f, "id:{}", pool_id),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::MockApi;
    use speculoos::prelude::*;

    #[test]
    fn test_pool_id_from_str() {
        let api = MockApi::default();
        let pool_id_str = "contract:cosmos1hsk6jryyqjfhp5dhc55tc9jtckygx0eph6dd02";
        let pool_id = UncheckedPoolId::from_str(pool_id_str).unwrap();
        let pool_id = pool_id.check(&api).unwrap();
        assert_that!(pool_id.to_string()).is_equal_to(pool_id_str.to_string());
    }

    #[test]
    fn test_expect_contract_happy() {
        let api = MockApi::default();
        let pool_id = PoolId::Contract(
            api.addr_validate("cosmos1hsk6jryyqjfhp5dhc55tc9jtckygx0eph6dd02")
                .unwrap(),
        );
        let res = pool_id.expect_contract();
        assert_that!(res).is_ok();
        assert_that!(res.unwrap()).is_equal_to(Addr::unchecked(
            "cosmos1hsk6jryyqjfhp5dhc55tc9jtckygx0eph6dd02",
        ));
    }

    #[test]
    fn test_expect_contract_sad() {
        let pool_id = PoolId::Id(1);
        let res = pool_id.expect_contract();
        assert_that!(res).is_err();
    }

    #[test]
    fn test_expect_id_happy() {
        let pool_id = PoolId::Id(1);
        let res = pool_id.expect_id();
        assert_that!(res).is_ok();
        assert_that!(res.unwrap()).is_equal_to(1);
    }

    #[test]
    fn test_expect_id_sad() {
        let api = MockApi::default();
        let pool_id = PoolId::Contract(
            api.addr_validate("cosmos1hsk6jryyqjfhp5dhc55tc9jtckygx0eph6dd02")
                .unwrap(),
        );
        let res = pool_id.expect_id();
        assert_that!(res).is_err();
    }
}
