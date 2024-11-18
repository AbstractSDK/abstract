use std::{
    fmt::{self, Display},
    str::FromStr,
};

use cosmwasm_std::{ensure_eq, to_json_binary, Addr, Binary, QuerierWrapper, StdError, StdResult};
use cw2::ContractVersion;
use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};
use semver::Version;

use super::module_reference::ModuleReference;
use crate::{
    error::AbstractError,
    objects::{fee::FixedFee, module_version::MODULE, namespace::Namespace},
    AbstractResult, IBC_CLIENT,
};

/// ID of the module
pub type ModuleId<'a> = &'a str;

/// Module status
#[cosmwasm_schema::cw_serde]
pub enum ModuleStatus {
    /// Modules in use
    Registered,
    /// Pending modules
    Pending,
    /// Yanked modules
    Yanked,
}

/// Stores the namespace, name, and version of an Abstract module.
#[cosmwasm_schema::cw_serde]
pub struct ModuleInfo {
    /// Namespace of the module
    pub namespace: Namespace,
    /// Name of the contract
    pub name: String,
    /// Version of the module
    pub version: ModuleVersion,
}

impl TryFrom<ModuleInfo> for ContractVersion {
    type Error = AbstractError;

    fn try_from(value: ModuleInfo) -> Result<Self, Self::Error> {
        let ModuleVersion::Version(version) = value.version else {
            return Err(AbstractError::MissingVersion("module".to_owned()));
        };
        Ok(ContractVersion {
            contract: format!("{}:{}", value.namespace, value.name),
            version,
        })
    }
}

const MAX_LENGTH: usize = 64;

/// Validate attributes of a [`ModuleInfo`].
/// We use the same conventions as Rust package names.
/// See <https://github.com/rust-lang/api-guidelines/discussions/29>
pub fn validate_name(name: &str) -> AbstractResult<()> {
    if name.is_empty() {
        return Err(AbstractError::FormattingError {
            object: "module name".into(),
            expected: "with content".into(),
            actual: "empty".to_string(),
        });
    }
    if name.len() > MAX_LENGTH {
        return Err(AbstractError::FormattingError {
            object: "module name".into(),
            expected: "at most 64 characters".into(),
            actual: name.len().to_string(),
        });
    }
    if name.contains(|c: char| !c.is_ascii_alphanumeric() && c != '-') {
        return Err(AbstractError::FormattingError {
            object: "module name".into(),
            expected: "alphanumeric characters and hyphens".into(),
            actual: name.to_string(),
        });
    }

    if name != name.to_lowercase() {
        return Err(AbstractError::FormattingError {
            object: "module name".into(),
            expected: name.to_ascii_lowercase(),
            actual: name.to_string(),
        });
    }
    Ok(())
}

impl ModuleInfo {
    pub fn from_id(id: &str, version: ModuleVersion) -> AbstractResult<Self> {
        let split: Vec<&str> = id.split(':').collect();
        if split.len() != 2 {
            return Err(AbstractError::FormattingError {
                object: "contract id".into(),
                expected: "namespace:contract_name".to_string(),
                actual: id.to_string(),
            });
        }
        Ok(ModuleInfo {
            namespace: Namespace::try_from(split[0])?,
            name: split[1].to_lowercase(),
            version,
        })
    }
    pub fn from_id_latest(id: &str) -> AbstractResult<Self> {
        Self::from_id(id, ModuleVersion::Latest)
    }

    pub fn validate(&self) -> AbstractResult<()> {
        self.namespace.validate()?;
        validate_name(&self.name)?;
        self.version.validate().map_err(|e| {
            StdError::generic_err(format!("Invalid version for module {}: {}", self.id(), e))
        })?;
        Ok(())
    }

    pub fn id(&self) -> String {
        format!("{}:{}", self.namespace, self.name)
    }

    pub fn id_with_version(&self) -> String {
        format!("{}:{}", self.id(), self.version)
    }

    pub fn assert_version_variant(&self) -> AbstractResult<()> {
        match &self.version {
            ModuleVersion::Latest => Err(AbstractError::Assert(
                "Module version must be set to a specific version".into(),
            )),
            ModuleVersion::Version(ver) => {
                // assert version parses correctly
                semver::Version::parse(ver)?;
                Ok(())
            }
        }
    }
}

impl<'a> PrimaryKey<'a> for &ModuleInfo {
    /// (namespace, name)
    type Prefix = (Namespace, String);

    /// namespace
    type SubPrefix = Namespace;

    /// version
    type Suffix = ModuleVersion;

    // (name, version)
    type SuperSuffix = (String, ModuleVersion);

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        let mut keys = self.namespace.key();
        keys.extend(self.name.key());
        keys.extend(self.version.key());
        keys
    }
}

impl<'a> Prefixer<'a> for &ModuleInfo {
    fn prefix(&self) -> Vec<Key> {
        let mut res = self.namespace.prefix();
        res.extend(self.name.prefix());
        res.extend(self.version.prefix());
        res
    }
}

impl KeyDeserialize for &ModuleInfo {
    type Output = ModuleInfo;
    const KEY_ELEMS: u16 = 1;

    #[inline(always)]
    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        let mut prov_name_ver = value.split_off(2);
        let prov_len = parse_length(&value)?;
        let mut len_name_ver = prov_name_ver.split_off(prov_len);

        let mut name_ver = len_name_ver.split_off(2);
        let ver_len = parse_length(&len_name_ver)?;
        let ver = name_ver.split_off(ver_len);

        Ok(ModuleInfo {
            namespace: Namespace::try_from(String::from_vec(prov_name_ver)?).map_err(|e| {
                StdError::generic_err(format!("Invalid namespace for module: {}", e))
            })?,
            name: String::from_vec(name_ver)?,
            version: ModuleVersion::from_vec(ver)?,
        })
    }
}

impl KeyDeserialize for ModuleVersion {
    type Output = ModuleVersion;
    const KEY_ELEMS: u16 = 1;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        let val = String::from_vec(value)?;
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

impl ModuleVersion {
    pub fn validate(&self) -> AbstractResult<()> {
        match &self {
            ModuleVersion::Latest => Ok(()),
            ModuleVersion::Version(ver) => {
                // assert version parses correctly
                Version::parse(ver)?;
                Ok(())
            }
        }
    }
}

impl FromStr for ModuleVersion {
    type Err = AbstractError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "latest" => Ok(Self::Latest),
            _ => {
                let v = Self::Version(s.to_owned());
                v.validate()?;
                Ok(v)
            }
        }
    }
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

impl<T> From<T> for ModuleVersion
where
    T: Into<String>,
{
    fn from(ver: T) -> Self {
        Self::Version(ver.into())
    }
}

impl fmt::Display for ModuleInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} provided by {} with version {}",
            self.name, self.namespace, self.version,
        )
    }
}

impl TryInto<Version> for ModuleVersion {
    type Error = AbstractError;

    fn try_into(self) -> AbstractResult<Version> {
        match self {
            ModuleVersion::Latest => Err(AbstractError::MissingVersion("module".to_string())),
            ModuleVersion::Version(ver) => {
                let version = Version::parse(&ver)?;
                Ok(version)
            }
        }
    }
}

impl<'a> PrimaryKey<'a> for ModuleVersion {
    type Prefix = ();

    type SubPrefix = ();

    type Suffix = Self;

    type SuperSuffix = Self;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        match &self {
            ModuleVersion::Latest => "latest".key(),
            ModuleVersion::Version(ver) => ver.key(),
        }
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

impl TryFrom<ContractVersion> for ModuleInfo {
    type Error = AbstractError;

    fn try_from(value: ContractVersion) -> Result<Self, Self::Error> {
        let split: Vec<&str> = value.contract.split(':').collect();
        if split.len() != 2 {
            return Err(AbstractError::FormattingError {
                object: "contract id".to_string(),
                expected: "namespace:contract_name".into(),
                actual: value.contract,
            });
        }
        Ok(ModuleInfo {
            namespace: Namespace::try_from(split[0])?,
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

impl From<(ModuleInfo, ModuleReference)> for Module {
    fn from((info, reference): (ModuleInfo, ModuleReference)) -> Self {
        Self { info, reference }
    }
}

impl Module {
    // Helper to know if this module supposed to be whitelisted on account contract
    pub fn should_be_whitelisted(&self) -> bool {
        match &self.reference {
            // Standalone, Service or Native(exception for IBC Client for the ICS20 Callbacks) contracts not supposed to be whitelisted on account
            ModuleReference::Adapter(_) | ModuleReference::App(_) => true,
            ModuleReference::Native(_) if self.info.id() == IBC_CLIENT => true,
            _ => false,
        }
    }
}

#[cosmwasm_schema::cw_serde]
pub struct ModuleInitMsg {
    pub fixed_init: Option<Binary>,
    pub owner_init: Option<Binary>,
}

impl ModuleInitMsg {
    pub fn format(self) -> AbstractResult<Binary> {
        match self {
            // If both set, receiving contract must handle it using the ModuleInitMsg
            ModuleInitMsg {
                fixed_init: Some(_),
                owner_init: Some(_),
            } => to_json_binary(&self),
            // If not, we can simplify by only sending the custom or fixed message.
            ModuleInitMsg {
                fixed_init: None,
                owner_init: Some(r),
            } => Ok(r),
            ModuleInitMsg {
                fixed_init: Some(f),
                owner_init: None,
            } => Ok(f),
            ModuleInitMsg {
                fixed_init: None,
                owner_init: None,
            } => Err(StdError::generic_err("No init msg set for this module")),
        }
        .map_err(Into::into)
    }
}

/// Assert that the provided module has the same data stored under the cw2 and module data keys.
pub fn assert_module_data_validity(
    querier: &QuerierWrapper,
    // The module that it claims to be
    module_claim: &Module,
    // Optional address, if not set, skip code_id checks
    module_address: Option<Addr>,
) -> AbstractResult<()> {
    // we retrieve the address information.
    let module_address = match &module_claim.reference.unwrap_addr() {
        Ok(addr) => addr.to_owned(),
        Err(..) => {
            // now we need to have a module address provided
            let Some(addr) = module_address else {
                // if no addr provided and module doesn't have it, just return
                // this will be the case when registering a code-id on Registry
                return Ok(());
            };
            addr
        }
    };

    let ModuleVersion::Version(version) = &module_claim.info.version else {
        panic!("Module version is not versioned, context setting is wrong")
    };

    // verify that the contract's data is equal to its registered data
    let cw_2_data_res = cw2::CONTRACT.query(querier, module_address.clone());

    // For standalone and service we only check the version if cw2 exists
    if let ModuleReference::Standalone(_) | ModuleReference::Service(_) = module_claim.reference {
        if let Ok(cw_2_data) = cw_2_data_res {
            ensure_eq!(
                version,
                &cw_2_data.version,
                AbstractError::UnequalModuleData {
                    cw2: cw_2_data.version,
                    module: version.to_owned()
                }
            );
        }
        return Ok(());
    }
    let cw_2_data = cw_2_data_res?;

    // Assert that the contract name is equal to the module name
    ensure_eq!(
        module_claim.info.id(),
        cw_2_data.contract,
        AbstractError::UnequalModuleData {
            cw2: cw_2_data.contract,
            module: module_claim.info.id()
        }
    );

    // Assert that the contract version is equal to the module version
    ensure_eq!(
        version,
        &cw_2_data.version,
        AbstractError::UnequalModuleData {
            cw2: cw_2_data.version,
            module: version.to_owned()
        }
    );
    // we're done if it's not an actual module
    match module_claim.reference {
        ModuleReference::Account(_) | ModuleReference::Native(_) | ModuleReference::Service(_) => {
            return Ok(())
        }
        _ => {}
    }

    let module_data = MODULE.query(querier, module_address)?;
    // assert that the names are equal
    ensure_eq!(
        module_data.module,
        cw_2_data.contract,
        AbstractError::UnequalModuleData {
            cw2: cw_2_data.contract,
            module: module_data.module,
        }
    );
    // assert that the versions are equal
    ensure_eq!(
        module_data.version,
        cw_2_data.version,
        AbstractError::UnequalModuleData {
            cw2: cw_2_data.version,
            module: module_data.version
        }
    );

    Ok(())
}

/// Module Monetization
#[cosmwasm_schema::cw_serde]
#[non_exhaustive]
pub enum Monetization {
    None,
    InstallFee(FixedFee),
}

impl Default for Monetization {
    fn default() -> Self {
        Self::None
    }
}

/// Module Metadata String
pub type ModuleMetadata = String;

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use cosmwasm_std::{testing::mock_dependencies, Addr, Order};
    use cw_storage_plus::Map;

    use super::*;

    mod storage_plus {
        use super::*;

        fn mock_key() -> ModuleInfo {
            ModuleInfo {
                namespace: Namespace::new("abstract").unwrap(),
                name: "rocket-ship".to_string(),
                version: ModuleVersion::Version("1.9.9".into()),
            }
        }

        fn mock_keys() -> (ModuleInfo, ModuleInfo, ModuleInfo, ModuleInfo) {
            (
                ModuleInfo {
                    namespace: Namespace::new("abstract").unwrap(),
                    name: "boat".to_string(),
                    version: ModuleVersion::Version("1.9.9".into()),
                },
                ModuleInfo {
                    namespace: Namespace::new("abstract").unwrap(),
                    name: "rocket-ship".to_string(),
                    version: ModuleVersion::Version("1.0.0".into()),
                },
                ModuleInfo {
                    namespace: Namespace::new("abstract").unwrap(),
                    name: "rocket-ship".to_string(),
                    version: ModuleVersion::Version("2.0.0".into()),
                },
                ModuleInfo {
                    namespace: Namespace::new("astroport").unwrap(),
                    name: "liquidity-pool".to_string(),
                    version: ModuleVersion::Version("10.5.7".into()),
                },
            )
        }

        #[coverage_helper::test]
        fn storage_key_works() {
            let mut deps = mock_dependencies();
            let key = mock_key();
            let map: Map<&ModuleInfo, u64> = Map::new("map");

            map.save(deps.as_mut().storage, &key, &42069).unwrap();

            assert_eq!(map.load(deps.as_ref().storage, &key).unwrap(), 42069);

            let items = map
                .range(deps.as_ref().storage, None, None, Order::Ascending)
                .map(|item| item.unwrap())
                .collect::<Vec<_>>();

            assert_eq!(items.len(), 1);
            assert_eq!(items[0], (key, 42069));
        }

        #[coverage_helper::test]
        fn storage_key_with_overlapping_name_namespace() {
            let mut deps = mock_dependencies();
            let info1 = ModuleInfo {
                namespace: Namespace::new("abstract").unwrap(),
                name: "ans".to_string(),
                version: ModuleVersion::Version("1.9.9".into()),
            };

            let _key1 = (&info1).joined_key();

            let info2 = ModuleInfo {
                namespace: Namespace::new("abs").unwrap(),
                name: "tractans".to_string(),
                version: ModuleVersion::Version("1.9.9".into()),
            };

            let _key2 = (&info2).joined_key();

            let map: Map<&ModuleInfo, u64> = Map::new("map");

            map.save(deps.as_mut().storage, &info1, &42069).unwrap();
            map.save(deps.as_mut().storage, &info2, &69420).unwrap();

            assert_eq!(
                map.keys_raw(&deps.storage, None, None, Order::Ascending)
                    .collect::<Vec<_>>()
                    .len(),
                2
            );
        }

        #[coverage_helper::test]
        fn composite_key_works() {
            let mut deps = mock_dependencies();
            let key = mock_key();
            let map: Map<(&ModuleInfo, Addr), u64> = Map::new("map");

            map.save(
                deps.as_mut().storage,
                (&key, Addr::unchecked("larry")),
                &42069,
            )
            .unwrap();

            map.save(
                deps.as_mut().storage,
                (&key, Addr::unchecked("jake")),
                &69420,
            )
            .unwrap();

            let items = map
                .prefix(&key)
                .range(deps.as_ref().storage, None, None, Order::Ascending)
                .map(|item| item.unwrap())
                .collect::<Vec<_>>();

            assert_eq!(items.len(), 2);
            assert_eq!(items[0], (Addr::unchecked("jake"), 69420));
            assert_eq!(items[1], (Addr::unchecked("larry"), 42069));
        }

        #[coverage_helper::test]
        fn partial_key_works() {
            let mut deps = mock_dependencies();
            let (key1, key2, key3, key4) = mock_keys();
            let map: Map<&ModuleInfo, u64> = Map::new("map");

            map.save(deps.as_mut().storage, &key1, &42069).unwrap();

            map.save(deps.as_mut().storage, &key2, &69420).unwrap();

            map.save(deps.as_mut().storage, &key3, &999).unwrap();

            map.save(deps.as_mut().storage, &key4, &13).unwrap();

            let items = map
                .sub_prefix(Namespace::new("abstract").unwrap())
                .range(deps.as_ref().storage, None, None, Order::Ascending)
                .map(|item| item.unwrap())
                .collect::<Vec<_>>();

            assert_eq!(items.len(), 3);
            assert_eq!(
                items[0],
                (
                    (
                        "boat".to_string(),
                        ModuleVersion::Version("1.9.9".to_string())
                    ),
                    42069
                )
            );
            assert_eq!(
                items[1],
                (
                    (
                        "rocket-ship".to_string(),
                        ModuleVersion::Version("1.0.0".to_string())
                    ),
                    69420
                )
            );

            assert_eq!(
                items[2],
                (
                    (
                        "rocket-ship".to_string(),
                        ModuleVersion::Version("2.0.0".to_string())
                    ),
                    999
                )
            );

            let items = map
                .sub_prefix(Namespace::new("astroport").unwrap())
                .range(deps.as_ref().storage, None, None, Order::Ascending)
                .map(|item| item.unwrap())
                .collect::<Vec<_>>();

            assert_eq!(items.len(), 1);
            assert_eq!(
                items[0],
                (
                    (
                        "liquidity-pool".to_string(),
                        ModuleVersion::Version("10.5.7".to_string())
                    ),
                    13
                )
            );
        }

        #[coverage_helper::test]
        fn partial_key_versions_works() {
            let mut deps = mock_dependencies();
            let (key1, key2, key3, key4) = mock_keys();
            let map: Map<&ModuleInfo, u64> = Map::new("map");

            map.save(deps.as_mut().storage, &key1, &42069).unwrap();

            map.save(deps.as_mut().storage, &key2, &69420).unwrap();

            map.save(deps.as_mut().storage, &key3, &999).unwrap();

            map.save(deps.as_mut().storage, &key4, &13).unwrap();

            let items = map
                .prefix((
                    Namespace::new("abstract").unwrap(),
                    "rocket-ship".to_string(),
                ))
                .range(deps.as_ref().storage, None, None, Order::Ascending)
                .map(|item| item.unwrap())
                .collect::<Vec<_>>();

            assert_eq!(items.len(), 2);
            assert_eq!(
                items[0],
                (ModuleVersion::Version("1.0.0".to_string()), 69420)
            );

            assert_eq!(items[1], (ModuleVersion::Version("2.0.0".to_string()), 999));
        }
    }

    mod module_info {
        use super::*;

        #[coverage_helper::test]
        fn validate_with_empty_name() {
            let info = ModuleInfo {
                namespace: Namespace::try_from("abstract").unwrap(),
                name: "".to_string(),
                version: ModuleVersion::Version("1.9.9".into()),
            };

            assert!(info.validate().unwrap_err().to_string().contains("empty"));
        }

        #[coverage_helper::test]
        fn validate_with_empty_namespace() {
            let info = ModuleInfo {
                namespace: Namespace::unchecked(""),
                name: "ans".to_string(),
                version: ModuleVersion::Version("1.9.9".into()),
            };

            assert!(info.validate().unwrap_err().to_string().contains("empty"));
        }

        use rstest::rstest;

        #[rstest]
        #[case("ans_host")]
        #[case("ans:host")]
        #[case("ans-host&")]
        fn validate_fails_with_non_alphanumeric(#[case] name: &str) {
            let info = ModuleInfo {
                namespace: Namespace::try_from("abstract").unwrap(),
                name: name.to_string(),
                version: ModuleVersion::Version("1.9.9".into()),
            };

            assert!(info
                .validate()
                .unwrap_err()
                .to_string()
                .contains("alphanumeric"));
        }

        #[rstest]
        #[case("lmao")]
        #[case("bad-")]
        fn validate_with_bad_versions(#[case] version: &str) {
            let info = ModuleInfo {
                namespace: Namespace::try_from("abstract").unwrap(),
                name: "ans".to_string(),
                version: ModuleVersion::Version(version.into()),
            };

            assert!(info
                .validate()
                .unwrap_err()
                .to_string()
                .contains("Invalid version"));
        }

        #[coverage_helper::test]
        fn id() {
            let info = ModuleInfo {
                name: "name".to_string(),
                namespace: Namespace::try_from("namespace").unwrap(),
                version: ModuleVersion::Version("1.0.0".into()),
            };

            let expected = "namespace:name".to_string();

            assert_eq!(info.id(), expected);
        }

        #[coverage_helper::test]
        fn id_with_version() {
            let info = ModuleInfo {
                name: "name".to_string(),
                namespace: Namespace::try_from("namespace").unwrap(),
                version: ModuleVersion::Version("1.0.0".into()),
            };

            let expected = "namespace:name:1.0.0".to_string();

            assert_eq!(info.id_with_version(), expected);
        }
    }

    mod module_version {
        use super::*;

        #[coverage_helper::test]
        fn try_into_version_happy_path() {
            let version = ModuleVersion::Version("1.0.0".into());

            let expected: Version = "1.0.0".to_string().parse().unwrap();

            let actual: Version = version.try_into().unwrap();

            assert_eq!(actual, expected);
        }

        #[coverage_helper::test]
        fn try_into_version_with_latest() {
            let version = ModuleVersion::Latest;

            let actual: Result<Version, _> = version.try_into();

            assert!(actual.is_err());
        }
    }

    mod standalone_modules_valid {
        use cosmwasm_std::testing::MOCK_CONTRACT_ADDR;

        use super::*;

        #[coverage_helper::test]
        fn no_cw2_contract() {
            let deps = mock_dependencies();
            let res = assert_module_data_validity(
                &deps.as_ref().querier,
                &Module {
                    info: ModuleInfo {
                        namespace: Namespace::new("counter").unwrap(),
                        name: "counter".to_owned(),
                        version: ModuleVersion::Version("1.1.0".to_owned()),
                    },
                    reference: ModuleReference::Standalone(0),
                },
                Some(Addr::unchecked(MOCK_CONTRACT_ADDR)),
            );
            assert!(res.is_ok());
        }
    }
}
