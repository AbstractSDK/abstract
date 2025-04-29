pub use abstract_std::ans_host::ExecuteMsgFns;
use abstract_std::{
    ans_host::*,
    objects::{
        pool_metadata::ResolvedPoolMetadata, AnsAsset, AnsEntryConvertor, AssetEntry, ChannelEntry,
        ContractEntry, DexAssetPairing, LpToken, PoolMetadata, PoolReference, UniquePoolId,
    },
    ANS_HOST,
};
use cosmwasm_std::Uint128;
use cw_address_like::AddressLike;
use cw_asset::{Asset, AssetInfo, AssetInfoBase, AssetInfoUnchecked, AssetUnchecked};
use cw_orch::{interface, prelude::*};

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct AnsHost<Chain>;

impl<Chain: CwEnv> cw_blob::interface::DeterministicInstantiation<Chain> for AnsHost<Chain> {}

impl<Chain: CwEnv> AnsHost<Chain> {
    pub fn balance(&self, addr: &Addr, asset: &AssetEntry) -> Result<Uint128, CwOrchError> {
        let asset: AssetInfo = self.resolve(asset)?;
        let chain = self.environment();
        match asset {
            AssetInfoBase::Native(denom) => chain
                .balance(addr, Some(denom))
                .map(|coins| coins[0].amount)
                .map_err(Into::into),
            AssetInfoBase::Cw20(cw20_addr) => {
                let res: cw20::BalanceResponse = chain
                    .query(
                        &cw20::Cw20QueryMsg::Balance {
                            address: addr.to_string(),
                        },
                        &cw20_addr,
                    )
                    .map_err(Into::into)?;
                Ok(res.balance)
            }
            _ => unimplemented!(),
        }
    }
}

impl<Chain: CwEnv> AnsHost<Chain> {
    pub fn resolve<R: ClientResolve<Chain>>(&self, item: &R) -> Result<R::Output, CwOrchError> {
        item.resolve(self)
    }
}

impl<Chain: CwEnv> Uploadable for AnsHost<Chain> {
    fn wrapper() -> <Mock as ::cw_orch::environment::TxHandler>::ContractSource {
        Box::new(
            ContractWrapper::new_with_empty(
                ::ans_host::contract::execute,
                ::ans_host::contract::instantiate,
                ::ans_host::contract::query,
            )
            .with_migrate(::ans_host::contract::migrate),
        )
    }
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("ans_host")
            .unwrap()
    }
}

impl<Chain: CwEnv> AnsHost<Chain>
where
    TxResponse<Chain>: IndexResponse,
{
    pub fn load(chain: Chain, address: &Addr) -> Self {
        let contract = cw_orch::contract::Contract::new(ANS_HOST, chain);
        contract.set_address(address);
        Self(contract)
    }
}

pub trait ClientResolve<Chain: CwEnv> {
    /// Result of resolving an entry.
    type Output;
    /// Resolve an entry into its value.
    fn resolve(&self, ans_host: &AnsHost<Chain>) -> Result<Self::Output, CwOrchError>;
}

// cw-multi-test doesn't support raw queries, so we will have to do smart queries instead

impl<Chain: CwEnv> ClientResolve<Chain> for AssetEntry {
    type Output = AssetInfo;

    fn resolve(&self, ans_host: &AnsHost<Chain>) -> Result<Self::Output, CwOrchError> {
        let mut assets: AssetsResponse = ans_host.query(&QueryMsg::Assets {
            names: vec![self.to_string()],
        })?;
        Ok(assets.assets.pop().unwrap().1)
    }
}

impl<Chain: CwEnv> ClientResolve<Chain> for LpToken {
    type Output = AssetInfo;

    fn resolve(&self, ans_host: &AnsHost<Chain>) -> Result<Self::Output, CwOrchError> {
        let asset_entry = AnsEntryConvertor::new(self.clone()).asset_entry();
        asset_entry.resolve(ans_host)
    }
}

impl<Chain: CwEnv> ClientResolve<Chain> for ContractEntry {
    type Output = Addr;

    fn resolve(&self, ans_host: &AnsHost<Chain>) -> Result<Self::Output, CwOrchError> {
        let mut contracts: ContractsResponse = ans_host.query(&QueryMsg::Contracts {
            entries: vec![self.clone()],
        })?;
        Ok(contracts.contracts.pop().unwrap().1)
    }
}

impl<Chain: CwEnv> ClientResolve<Chain> for ChannelEntry {
    type Output = String;

    fn resolve(&self, ans_host: &AnsHost<Chain>) -> Result<Self::Output, CwOrchError> {
        let mut channels: ChannelsResponse = ans_host.query(&QueryMsg::Channels {
            entries: vec![self.clone()],
        })?;
        Ok(channels.channels.pop().unwrap().1)
    }
}

impl<Chain: CwEnv> ClientResolve<Chain> for DexAssetPairing {
    type Output = Vec<PoolReference>;

    fn resolve(&self, ans_host: &AnsHost<Chain>) -> Result<Self::Output, CwOrchError> {
        let mut pool_address_list: PoolAddressListResponse =
            ans_host.query(&QueryMsg::PoolList {
                filter: Some(AssetPairingFilter {
                    asset_pair: Some((self.asset_x().clone(), self.asset_y().clone())),
                    dex: Some(self.dex().to_owned()),
                }),
                start_after: None,
                limit: None,
            })?;
        let found = pool_address_list
            .pools
            .pop()
            .ok_or(CwOrchError::StdErr(format!(
                "Pool reference for {self} not found"
            )))?;
        Ok(found.1.clone())
    }
}

impl<Chain: CwEnv> ClientResolve<Chain> for UniquePoolId {
    type Output = PoolMetadata;

    fn resolve(&self, ans_host: &AnsHost<Chain>) -> Result<Self::Output, CwOrchError> {
        let mut pool_metadatas: PoolMetadatasResponse =
            ans_host.query(&QueryMsg::PoolMetadatas { ids: vec![*self] })?;
        Ok(pool_metadatas.metadatas.pop().unwrap().1)
    }
}

impl<Chain: CwEnv> ClientResolve<Chain> for AnsAsset {
    type Output = Asset;

    fn resolve(&self, ans_host: &AnsHost<Chain>) -> Result<Self::Output, CwOrchError> {
        Ok(Asset::new(self.name.resolve(ans_host)?, self.amount))
    }
}

impl<Chain: CwEnv, T: AddressLike> ClientResolve<Chain> for AssetInfoBase<T> {
    type Output = AssetEntry;

    fn resolve(&self, ans_host: &AnsHost<Chain>) -> Result<Self::Output, CwOrchError> {
        let asset_info_unchecked = match self {
            AssetInfoBase::Native(native) => AssetInfoUnchecked::native(native),
            AssetInfoBase::Cw20(cw20) => AssetInfoUnchecked::cw20(cw20.to_string()),
            _ => panic!("Not supported asset type"),
        };
        let mut assets: AssetInfosResponse = ans_host.query(&QueryMsg::AssetInfos {
            infos: vec![asset_info_unchecked],
        })?;
        Ok(assets.infos.pop().unwrap().1)
    }
}

impl<Chain: CwEnv> ClientResolve<Chain> for AssetUnchecked {
    type Output = AnsAsset;

    fn resolve(&self, ans_host: &AnsHost<Chain>) -> Result<Self::Output, CwOrchError> {
        Ok(AnsAsset {
            name: self.info.resolve(ans_host)?,
            amount: self.amount,
        })
    }
}

impl<Chain: CwEnv> ClientResolve<Chain> for PoolMetadata {
    type Output = ResolvedPoolMetadata;

    fn resolve(&self, ans_host: &AnsHost<Chain>) -> Result<Self::Output, CwOrchError> {
        Ok(ResolvedPoolMetadata {
            assets: self.assets.resolve(ans_host)?,
            dex: self.dex.clone(),
            pool_type: self.pool_type,
        })
    }
}

impl<Chain: CwEnv, T> ClientResolve<Chain> for Vec<T>
where
    T: ClientResolve<Chain>,
{
    type Output = Vec<T::Output>;

    fn resolve(&self, ans_host: &AnsHost<Chain>) -> Result<Self::Output, CwOrchError> {
        self.iter().map(|entry| entry.resolve(ans_host)).collect()
    }
}
