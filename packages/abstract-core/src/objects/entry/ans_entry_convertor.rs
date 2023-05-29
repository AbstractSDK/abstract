use crate::constants::{ASSET_DELIMITER, TYPE_DELIMITER};
use crate::objects::{AssetEntry, DexAssetPairing, LpToken, PoolMetadata};

use crate::AbstractResult;

/// A helper struct for Abstract Name Service entry conversions.
pub struct AnsEntryConvertor<T> {
    entry: T,
}

impl<T> AnsEntryConvertor<T> {
    pub fn new(entry: T) -> Self {
        Self { entry }
    }
}

// An LP token can convert to:
impl AnsEntryConvertor<LpToken> {
    pub fn asset_entry(self) -> AssetEntry {
        AssetEntry::from(self.entry.to_string())
    }

    pub fn dex_asset_pairing(self) -> AbstractResult<DexAssetPairing> {
        let mut assets = self.entry.assets;
        // assets should already be sorted, but just in case
        assets.sort();
        assets.reverse();

        Ok(DexAssetPairing::new(
            assets.pop().unwrap(),
            assets.pop().unwrap(),
            self.entry.dex.as_str(),
        ))
    }
}

impl AnsEntryConvertor<PoolMetadata> {
    pub fn lp_token(self) -> LpToken {
        LpToken {
            dex: self.entry.dex,
            assets: self.entry.assets,
        }
    }
}

impl AnsEntryConvertor<AssetEntry> {
    /// Try from an asset entry that should be formatted as "dex_name/asset1,asset2"
    pub fn lp_token(self) -> AbstractResult<LpToken> {
        let segments = self
            .entry
            .as_str()
            .split(TYPE_DELIMITER)
            .collect::<Vec<_>>();

        if segments.len() != 2 {
            return Err(crate::AbstractError::FormattingError {
                object: "lp token".to_string(),
                expected: "type/asset1,asset2".to_string(),
                actual: self.entry.to_string(),
            });
        }

        // get the dex name, like "junoswap"
        let dex_name = segments[0].to_string();

        // get the assets, like "crab,junox" and split them
        let mut assets: Vec<AssetEntry> = segments[1]
            .split(ASSET_DELIMITER)
            .map(AssetEntry::from)
            .collect();

        // sort the assets on name
        assets.sort_unstable();

        if assets.len() < 2 {
            return Err(crate::AbstractError::FormattingError {
                object: "lp token".into(),
                expected: "at least 2 assets in LP token".into(),
                actual: self.entry.to_string(),
            });
        }

        Ok(LpToken {
            dex: dex_name,
            assets,
        })
    }
}
