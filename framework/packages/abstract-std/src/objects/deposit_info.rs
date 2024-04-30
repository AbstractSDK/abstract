use cw_asset::AssetInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{error::AbstractError, AbstractResult};

/// Helper for handling deposit assets.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositInfo {
    pub asset_info: AssetInfo,
}

impl DepositInfo {
    pub fn assert(&self, asset_info: &AssetInfo) -> AbstractResult<()> {
        if asset_info == &self.asset_info {
            return Ok(());
        }

        Err(AbstractError::Assert(format!(
            "Invalid deposit asset. Expected {}, got {}.",
            self.asset_info, asset_info
        )))
    }

    pub fn get_denom(self) -> AbstractResult<String> {
        match self.asset_info {
            AssetInfo::Native(denom) => Ok(denom),
            AssetInfo::Cw20(..) => Err(AbstractError::Assert(
                "'denom' only exists for native tokens.".into(),
            )),
            _ => panic!("asset not supported"),
        }
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::Addr;

    use super::*;

    pub const TEST_DENOM1: &str = "uusd";
    pub const TEST_DENOM2: &str = "uluna";
    pub const TEST_ADDR1: &str = "1234";
    pub const TEST_ADDR2: &str = "4321";

    #[test]
    fn test_failing_assert_for_native_tokens() {
        let deposit_info = DepositInfo {
            asset_info: AssetInfo::Native(TEST_DENOM1.to_string()),
        };
        let other_native_token = AssetInfo::Native(TEST_DENOM2.to_string());
        assert!(deposit_info.assert(&other_native_token).is_err());
    }

    #[test]
    fn test_passing_assert_for_native_tokens() {
        let deposit_info = DepositInfo {
            asset_info: AssetInfo::Native(TEST_DENOM1.to_string()),
        };
        let other_native_token = AssetInfo::Native(TEST_DENOM1.to_string());
        assert!(deposit_info.assert(&other_native_token).is_ok());
    }

    #[test]
    fn test_failing_assert_for_nonnative_tokens() {
        let deposit_info = DepositInfo {
            asset_info: AssetInfo::Cw20(Addr::unchecked(TEST_ADDR1.to_string())),
        };
        let other_native_token = AssetInfo::Cw20(Addr::unchecked(TEST_ADDR2.to_string()));
        assert!(deposit_info.assert(&other_native_token).is_err());
    }

    #[test]
    fn test_passing_assert_for_nonnative_tokens() {
        let deposit_info = DepositInfo {
            asset_info: AssetInfo::Cw20(Addr::unchecked(TEST_ADDR1.to_string())),
        };
        let other_native_token = AssetInfo::Cw20(Addr::unchecked(TEST_ADDR1.to_string()));
        assert!(deposit_info.assert(&other_native_token).is_ok());
    }

    #[test]
    fn test_failing_assert_for_mixed_tokens() {
        let deposit_info = DepositInfo {
            asset_info: AssetInfo::Native(TEST_DENOM1.to_string()),
        };
        let other_native_token = AssetInfo::Cw20(Addr::unchecked(TEST_DENOM1.to_string()));
        assert!(deposit_info.assert(&other_native_token).is_err());
    }
}
