use crate::{error::AbstractError, AbstractResult};
use cosmwasm_std::{Addr, Api, StdError};
use std::{fmt, str::FromStr};

#[cosmwasm_schema::cw_serde]
#[non_exhaustive]
pub enum PoolAddressBase<T> {
    Contract(T),
    Id(u64),
}

impl<T> PoolAddressBase<T> {
    pub fn contract<C: Into<T>>(contract: C) -> Self {
        Self::Contract(contract.into())
    }
    pub fn id<N: Into<u64>>(id: N) -> Self {
        Self::Id(id.into())
    }
}

/// Actual instance of a PoolAddress with verified data
pub type PoolAddress = PoolAddressBase<Addr>;

impl PoolAddress {
    pub fn expect_contract(&self) -> AbstractResult<Addr> {
        match self {
            PoolAddress::Contract(addr) => Ok(addr.clone()),
            _ => Err(AbstractError::Assert(
                "Pool address not a contract address.".into(),
            )),
        }
    }

    pub fn expect_id(&self) -> AbstractResult<u64> {
        match self {
            PoolAddress::Id(id) => Ok(*id),
            _ => Err(AbstractError::Assert(
                "Pool address not an numerical ID.".into(),
            )),
        }
    }
}
/// Instance of a PoolAddress passed around messages
pub type UncheckedPoolAddress = PoolAddressBase<String>;

impl FromStr for UncheckedPoolAddress {
    type Err = AbstractError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let words: Vec<&str> = s.split(':').collect();

        match words[0] {
            "contract" => {
                if words.len() != 2 {
                    return Err(AbstractError::FormattingError {
                        object: "unchecked pool address".to_string(),
                        expected: "contract:{{contract_addr}}".to_string(),
                        actual: s.to_string(),
                    });
                }

                Ok(UncheckedPoolAddress::Contract(String::from(words[1])))
            }
            "id" => {
                if words.len() != 2 {
                    return Err(AbstractError::FormattingError {
                        object: "unchecked pool address".to_string(),
                        expected: "id:{{pool_id}}".to_string(),
                        actual: s.to_string(),
                    });
                }
                let parsed_id_res = words[1].parse::<u64>();
                match parsed_id_res {
                    Ok(id) => Ok(UncheckedPoolAddress::Id(id)),
                    Err(err) => Err(StdError::generic_err(err.to_string()).into()),
                }
            }
            _unknown => Err(AbstractError::FormattingError {
                object: "unchecked pool address".to_string(),
                expected: "'contract' or 'id'".to_string(),
                actual: s.to_string(),
            }),
        }
    }
}

impl From<PoolAddress> for UncheckedPoolAddress {
    fn from(pool_info: PoolAddress) -> Self {
        match pool_info {
            PoolAddress::Contract(contract_addr) => {
                UncheckedPoolAddress::Contract(contract_addr.into())
            }
            PoolAddress::Id(denom) => UncheckedPoolAddress::Id(denom),
        }
    }
}

impl From<&PoolAddress> for UncheckedPoolAddress {
    fn from(pool_id: &PoolAddress) -> Self {
        match pool_id {
            PoolAddress::Contract(contract_addr) => {
                UncheckedPoolAddress::Contract(contract_addr.into())
            }
            PoolAddress::Id(denom) => UncheckedPoolAddress::Id(*denom),
        }
    }
}

impl From<Addr> for PoolAddress {
    fn from(contract_addr: Addr) -> Self {
        PoolAddress::Contract(contract_addr)
    }
}

impl UncheckedPoolAddress {
    /// Validate data contained in an _unchecked_ **pool id** instance; return a new _checked_
    /// **pool id** instance:
    /// * For Contract addresses, assert its address is valid
    ///
    ///
    /// ```rust,no_run
    /// use cosmwasm_std::{Addr, Api};
    /// use abstract_core::{objects::pool_id::UncheckedPoolAddress, AbstractResult};
    ///
    /// fn validate_pool_id(api: &dyn Api, pool_id_unchecked: &UncheckedPoolAddress) {
    ///     match pool_id_unchecked.check(api) {
    ///         Ok(info) => println!("pool id is valid: {}", info.to_string()),
    ///         Err(err) => println!("pool id is invalid! reason: {}", err),
    ///     }
    /// }
    /// ```
    pub fn check(&self, api: &dyn Api) -> AbstractResult<PoolAddress> {
        Ok(match self {
            UncheckedPoolAddress::Contract(contract_addr) => {
                PoolAddress::Contract(api.addr_validate(contract_addr)?)
            }
            UncheckedPoolAddress::Id(pool_id) => PoolAddress::Id(*pool_id),
        })
    }
}

impl fmt::Display for PoolAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PoolAddress::Contract(contract_addr) => write!(f, "contract:{contract_addr}"),
            PoolAddress::Id(pool_id) => write!(f, "id:{pool_id}"),
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
        let pool_id = UncheckedPoolAddress::from_str(pool_id_str).unwrap();
        let pool_id = pool_id.check(&api).unwrap();
        assert_that!(pool_id.to_string()).is_equal_to(pool_id_str.to_string());
    }

    #[test]
    fn test_expect_contract_happy() {
        let api = MockApi::default();
        let pool_id = PoolAddress::Contract(
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
        let pool_id = PoolAddress::Id(1);
        let res = pool_id.expect_contract();
        assert_that!(res).is_err();
    }

    #[test]
    fn test_expect_id_happy() {
        let pool_id = PoolAddress::Id(1);
        let res = pool_id.expect_id();
        assert_that!(res).is_ok();
        assert_that!(res.unwrap()).is_equal_to(1);
    }

    #[test]
    fn test_expect_id_sad() {
        let api = MockApi::default();
        let pool_id = PoolAddress::Contract(
            api.addr_validate("cosmos1hsk6jryyqjfhp5dhc55tc9jtckygx0eph6dd02")
                .unwrap(),
        );
        let res = pool_id.expect_id();
        assert_that!(res).is_err();
    }
}
