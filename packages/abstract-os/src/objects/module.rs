use std::fmt::{self, Display};

use cosmwasm_std::{to_binary, Binary, StdError, StdResult};
use cw2::ContractVersion;
use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};

use super::module_reference::ModuleReference;

/// Stores the provider, name, and version of an Abstract module.
#[cosmwasm_schema::cw_serde]
pub struct ModuleInfo {
    /// Provider of the module
    pub provider: String,
    /// Name of the contract
    pub name: String,
    /// Version of the module
    pub version: ModuleVersion,
}

impl ModuleInfo {
    pub fn id(&self) -> String {
        format!("{}:{}", self.provider, self.name)
    }
    pub fn from_id(id: &str, version: ModuleVersion) -> StdResult<Self> {
        let split: Vec<&str> = id.split(':').collect();
        if split.len() != 2 {
            return Err(StdError::generic_err(format!(
                "contract id:{} must be formatted as provider:contract_name.",
                id
            )));
        }
        Ok(ModuleInfo {
            provider: split[0].to_lowercase(),
            name: split[1].to_lowercase(),
            version,
        })
    }

    pub fn from_id_latest(id: &str) -> StdResult<Self> {
        Self::from_id(id, ModuleVersion::Latest)
    }

    pub fn assert_version_variant(&self) -> StdResult<()> {
        match &self.version {
            ModuleVersion::Latest => Err(StdError::generic_err(
                "Module version must be set for this action.",
            )),
            ModuleVersion::Version(ver) => {
                // assert version parses correctly
                semver::Version::parse(ver).map_err(|e|StdError::generic_err(e.to_string()))?;
                Ok(())
            },
        }
    }
}

impl<'a> PrimaryKey<'a> for ModuleInfo {
    type Prefix = (String, String);

    type SubPrefix = String;

    /// Possibly change to ModuleVersion in future by implementing PrimaryKey
    type Suffix = String;

    type SuperSuffix = (String, String);

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        let mut keys = self.provider.key();
        keys.extend(self.name.key());
        let temp = match &self.version {
            ModuleVersion::Latest => "latest".key(),
            ModuleVersion::Version(ver) => ver.key(),
        };
        keys.extend(temp);
        keys
    }
}

impl<'a> Prefixer<'a> for ModuleInfo {
    fn prefix(&self) -> Vec<Key> {
        let mut res = self.provider.prefix();
        res.extend(self.name.prefix().into_iter());
        res.extend(self.version.prefix().into_iter());
        res
    }
}

impl<'a> Prefixer<'a> for ModuleVersion {
    fn prefix(&self) -> Vec<Key> {
        let self_as_bytes = match &self {
            ModuleVersion::Latest => "latest".as_bytes(),
            ModuleVersion::Version(ver) => ver.as_bytes(),
        };
        vec![Key::Ref(self_as_bytes)]
    }
}

impl KeyDeserialize for ModuleInfo {
    type Output = Self;

    #[inline(always)]
    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        let mut prov_name_ver = value.split_off(2);
        let prov_len = parse_length(&value)?;
        let mut len_name_ver = prov_name_ver.split_off(prov_len);

        let mut name_ver = len_name_ver.split_off(2);
        let ver_len = parse_length(&len_name_ver)?;
        let ver = name_ver.split_off(ver_len);

        Ok(Self {
            provider: String::from_vec(prov_name_ver)?,
            name: String::from_vec(name_ver)?,
            version: ModuleVersion::from_vec(ver)?,
        })
    }
}

impl KeyDeserialize for ModuleVersion {
    type Output = Self;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        let val = String::from_utf8(value).map_err(StdError::invalid_utf8)?;
        if &val == "latest" {
            Ok(Self::Latest)
        } else {
            Ok(Self::Version(val))
        }
    }
}

#[inline(always)]
fn parse_length(value: &[u8]) -> StdResult<usize> {
    Ok(u16::from_be_bytes(
        value
            .try_into()
            .map_err(|_| StdError::generic_err("Could not read 2 byte length"))?,
    )
    .into())
}

#[cosmwasm_schema::cw_serde]
pub enum ModuleVersion {
    Latest,
    Version(String),
}

// Do not change!!
impl Display for ModuleVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let print_str = match self {
            ModuleVersion::Latest => "latest".to_string(),
            ModuleVersion::Version(ver) => ver.to_owned(),
        };
        f.write_str(&print_str)
    }
}

impl<T> From<T> for ModuleVersion where T: Into<String>{
    fn from(ver: T) -> Self {
        Self::Version(ver.into())
    }
}

impl fmt::Display for ModuleInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} provided by {} with version {}",
            self.name, self.provider, self.version,
        )
    }
}

impl TryFrom<ContractVersion> for ModuleInfo {
    type Error = StdError;

    fn try_from(value: ContractVersion) -> Result<Self, Self::Error> {
        let split: Vec<&str> = value.contract.split(':').collect();
        if split.len() != 2 {
            return Err(StdError::generic_err(format!(
                "contract id:{} must be formatted as provider:contract_name.",
                value.contract
            )));
        }
        Ok(ModuleInfo {
            provider: split[0].to_lowercase(),
            name: split[1].to_lowercase(),
            version: ModuleVersion::Version(value.version),
        })
    }
}

#[cosmwasm_schema::cw_serde]

pub struct Module {
    pub info: ModuleInfo,
    pub reference: ModuleReference,
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "info: {}, reference: {:?}", self.info, self.reference)
    }
}

#[cosmwasm_schema::cw_serde]

pub struct ModuleInitMsg {
    pub fixed_init: Option<Binary>,
    pub root_init: Option<Binary>,
}

impl ModuleInitMsg {
    pub fn format(self) -> StdResult<Binary> {
        match self {
            // If both set, receiving contract must handle it using the ModuleInitMsg
            ModuleInitMsg {
                fixed_init: Some(_),
                root_init: Some(_),
            } => to_binary(&self),
            // If not, we can simplify by only sending the custom or fixed message.
            ModuleInitMsg {
                fixed_init: None,
                root_init: Some(r),
            } => Ok(r),
            ModuleInitMsg {
                fixed_init: Some(f),
                root_init: None,
            } => Ok(f),
            ModuleInitMsg {
                fixed_init: None,
                root_init: None,
            } => Err(StdError::generic_err("No init msg set for this module")),
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::{testing::mock_dependencies, Addr, Order};
    use cw_storage_plus::Map;

    fn mock_key() -> ModuleInfo {
        ModuleInfo {
            provider: "abstract".to_string(),
            name: "rocket-ship".to_string(),
            version: ModuleVersion::Version("1.9.9".into()),
        }
    }

    fn mock_keys() -> (ModuleInfo, ModuleInfo, ModuleInfo, ModuleInfo) {
        (
            ModuleInfo {
                provider: "abstract".to_string(),
                name: "boat".to_string(),
                version: ModuleVersion::Version("1.9.9".into()),
            },
            ModuleInfo {
                provider: "abstract".to_string(),
                name: "rocket-ship".to_string(),
                version: ModuleVersion::Version("1.0.0".into()),
            },
            ModuleInfo {
                provider: "abstract".to_string(),
                name: "rocket-ship".to_string(),
                version: ModuleVersion::Version("2.0.0".into()),
            },
            ModuleInfo {
                provider: "astroport".to_string(),
                name: "liquidity_pool".to_string(),
                version: ModuleVersion::Latest,
            },
        )
    }

    #[test]
    fn storage_key_works() {
        let mut deps = mock_dependencies();
        let key = mock_key();
        let map: Map<ModuleInfo, u64> = Map::new("map");

        map.save(deps.as_mut().storage, key.clone(), &42069)
            .unwrap();

        assert_eq!(map.load(deps.as_ref().storage, key.clone()).unwrap(), 42069);

        let items = map
            .range(deps.as_ref().storage, None, None, Order::Ascending)
            .map(|item| item.unwrap())
            .collect::<Vec<_>>();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0], (key, 42069));
    }

    #[test]
    fn composite_key_works() {
        let mut deps = mock_dependencies();
        let key = mock_key();
        let map: Map<(ModuleInfo, Addr), u64> = Map::new("map");

        map.save(
            deps.as_mut().storage,
            (key.clone(), Addr::unchecked("larry")),
            &42069,
        )
        .unwrap();

        map.save(
            deps.as_mut().storage,
            (key.clone(), Addr::unchecked("jake")),
            &69420,
        )
        .unwrap();

        let items = map
            .prefix(key)
            .range(deps.as_ref().storage, None, None, Order::Ascending)
            .map(|item| item.unwrap())
            .collect::<Vec<_>>();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0], (Addr::unchecked("jake"), 69420));
        assert_eq!(items[1], (Addr::unchecked("larry"), 42069));
    }

    #[test]
    fn partial_key_works() {
        let mut deps = mock_dependencies();
        let (key1, key2, key3, key4) = mock_keys();
        let map: Map<ModuleInfo, u64> = Map::new("map");

        map.save(deps.as_mut().storage, key1, &42069).unwrap();

        map.save(deps.as_mut().storage, key2, &69420).unwrap();

        map.save(deps.as_mut().storage, key3, &999).unwrap();

        map.save(deps.as_mut().storage, key4, &13).unwrap();

        let items = map
            .sub_prefix("abstract".to_string())
            .range(deps.as_ref().storage, None, None, Order::Ascending)
            .map(|item| item.unwrap())
            .collect::<Vec<_>>();

        assert_eq!(items.len(), 3);
        assert_eq!(items[0], (("boat".to_string(), "1.9.9".to_string()), 42069));
        assert_eq!(
            items[1],
            (("rocket-ship".to_string(), "1.0.0".to_string()), 69420)
        );

        assert_eq!(
            items[2],
            (("rocket-ship".to_string(), "2.0.0".to_string()), 999)
        );

        let items = map
            .sub_prefix("astroport".to_string())
            .range(deps.as_ref().storage, None, None, Order::Ascending)
            .map(|item| item.unwrap())
            .collect::<Vec<_>>();

        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0],
            (("liquidity_pool".to_string(), "latest".to_string()), 13)
        );
    }

    #[test]
    fn partial_key_versions_works() {
        let mut deps = mock_dependencies();
        let (key1, key2, key3, key4) = mock_keys();
        let map: Map<ModuleInfo, u64> = Map::new("map");

        map.save(deps.as_mut().storage, key1, &42069).unwrap();

        map.save(deps.as_mut().storage, key2, &69420).unwrap();

        map.save(deps.as_mut().storage, key3, &999).unwrap();

        map.save(deps.as_mut().storage, key4, &13).unwrap();

        let items = map
            .prefix(("abstract".to_string(), "rocket-ship".to_string()))
            .range(deps.as_ref().storage, None, None, Order::Ascending)
            .map(|item| item.unwrap())
            .collect::<Vec<_>>();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0], ("1.0.0".to_string(), 69420));

        assert_eq!(items[1], ("2.0.0".to_string(), 999));
    }
}
