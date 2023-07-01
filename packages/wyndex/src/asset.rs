use cosmwasm_schema::cw_serde;
use cosmwasm_std::Env;
use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};
use std::fmt;

use crate::pair::PairInfo;
use crate::pair::QueryMsg as PairQueryMsg;
use crate::querier::{
    query_balance, query_token_balance, query_token_symbol, NATIVE_TOKEN_PRECISION,
};
use cosmwasm_std::{
    to_binary, Addr, Api, BankMsg, Coin, ConversionOverflowError, CosmosMsg, Decimal256, Fraction,
    MessageInfo, QuerierWrapper, StdError, StdResult, Uint128, Uint256, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20QueryMsg, MinterResponse, TokenInfoResponse};
use itertools::Itertools;

/// Minimum initial LP share
pub const MINIMUM_LIQUIDITY_AMOUNT: Uint128 = Uint128::new(1_000);

/// This enum describes a Terra asset (native or CW20).
#[cw_serde]
pub struct Asset {
    /// Information about an asset stored in a [`AssetInfo`] struct
    pub info: AssetInfo,
    /// A token amount
    pub amount: Uint128,
}

impl Asset {
    /// Returns true if the token is native. Otherwise returns false.
    pub fn is_native_token(&self) -> bool {
        self.info.is_native_token()
    }

    /// Checks that the tokens' denom or contract addr is lowercased and valid.
    pub fn validate(&self, api: &dyn Api) -> StdResult<AssetValidated> {
        Ok(AssetValidated {
            info: self.info.validate(api)?,
            amount: self.amount,
        })
    }
}

/// This enum describes an asset (native or CW20).
#[cw_serde]
pub struct AssetValidated {
    /// Information about an asset stored in a [`AssetInfoValidated`] struct
    pub info: AssetInfoValidated,
    /// A token amount
    pub amount: Uint128,
}

impl From<AssetValidated> for Asset {
    fn from(asset: AssetValidated) -> Self {
        Asset {
            info: asset.info.into(),
            amount: asset.amount,
        }
    }
}
impl From<&AssetValidated> for Asset {
    fn from(asset: &AssetValidated) -> Self {
        Asset {
            info: asset.info.clone().into(),
            amount: asset.amount,
        }
    }
}

/// This struct describes a Terra asset as decimal.
#[cw_serde]
pub struct DecimalAsset {
    pub info: AssetInfoValidated,
    pub amount: Decimal256,
}

impl fmt::Display for AssetValidated {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.amount, self.info)
    }
}

impl AssetValidated {
    /// Returns true if the token is native. Otherwise returns false.
    pub fn is_native_token(&self) -> bool {
        self.info.is_native_token()
    }

    /// For native tokens of type [`AssetInfo`] uses the default method [`BankMsg::Send`] to send a token amount to a recipient.
    /// Before the token is sent, we need to deduct a tax.
    ///
    /// For a token of type [`AssetInfo`] we use the default method [`Cw20ExecuteMsg::Transfer`] and so there's no need to deduct any other tax.
    pub fn into_msg(&self, recipient: impl Into<String>) -> StdResult<CosmosMsg> {
        let recipient = recipient.into();
        match &self.info {
            AssetInfoValidated::Token(contract_addr) => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient,
                    amount: self.amount,
                })?,
                funds: vec![],
            })),
            AssetInfoValidated::Native(denom) => Ok(CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient,
                amount: vec![Coin {
                    denom: denom.to_string(),
                    amount: self.amount,
                }],
            })),
        }
    }

    /// For native coins, this asserts that they were received with this message already.
    /// For cw20 tokens, this adds a transfer message to the given `Vec` to receive them.
    pub fn receive(
        &self,
        env: &Env,
        info: &MessageInfo,
        messages: &mut Vec<CosmosMsg>,
    ) -> StdResult<()> {
        if self.amount.is_zero() {
            return Ok(());
        }

        match &self.info {
            AssetInfoValidated::Native(_) => self.assert_sent_native_token_balance(info),
            AssetInfoValidated::Token(contract_addr) => {
                messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: contract_addr.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                        owner: info.sender.to_string(),
                        recipient: env.contract.address.to_string(),
                        amount: self.amount,
                    })?,
                    funds: vec![],
                }));
                Ok(())
            }
        }
    }

    /// Validates an amount of native tokens being sent.
    pub fn assert_sent_native_token_balance(&self, message_info: &MessageInfo) -> StdResult<()> {
        if let AssetInfoValidated::Native(denom) = &self.info {
            match message_info.funds.iter().find(|x| x.denom == *denom) {
                Some(coin) => {
                    if self.amount == coin.amount {
                        Ok(())
                    } else {
                        Err(StdError::generic_err("Native token balance mismatch between the argument and the transferred"))
                    }
                }
                None => {
                    if self.amount.is_zero() {
                        Ok(())
                    } else {
                        Err(StdError::generic_err("Native token balance mismatch between the argument and the transferred"))
                    }
                }
            }
        } else {
            Ok(())
        }
    }

    pub fn to_decimal_asset(&self, precision: impl Into<u32>) -> StdResult<DecimalAsset> {
        Ok(DecimalAsset {
            info: self.info.clone(),
            amount: Decimal256::with_precision(self.amount, precision.into())?,
        })
    }
}

#[cw_serde]
#[derive(Eq, Hash)]
pub enum AssetInfo {
    /// Non-native Token
    Token(String),
    /// Native token
    Native(String),
}

impl AssetInfo {
    /// Returns true if the caller is a native token. Otherwise returns false.
    pub fn is_native_token(&self) -> bool {
        matches!(self, AssetInfo::Native(_))
    }

    /// Checks that the tokens' denom or contract addr is lowercased and valid.
    pub fn validate(&self, api: &dyn Api) -> StdResult<AssetInfoValidated> {
        Ok(match self {
            AssetInfo::Token(contract_addr) => {
                AssetInfoValidated::Token(api.addr_validate(contract_addr.as_str())?)
            }
            AssetInfo::Native(denom) => {
                if !denom.starts_with("ibc/") && denom != &denom.to_lowercase() {
                    return Err(StdError::generic_err(format!(
                        "Non-IBC token denom {} should be lowercase",
                        denom
                    )));
                }
                AssetInfoValidated::Native(denom.to_string())
            }
        })
    }
    pub fn query_pool(
        &self,
        querier: &QuerierWrapper,
        pool_addr: impl Into<String>,
    ) -> StdResult<Uint128> {
        match self {
            AssetInfo::Token(contract_addr) => {
                query_token_balance(querier, contract_addr, pool_addr)
            }
            AssetInfo::Native(denom) => query_balance(querier, pool_addr, denom),
        }
    }

    /// If the caller object is a native token of type [`AssetInfo`] then his `denom` field converts to a byte string.
    ///
    /// If the caller object is a token of type [`AssetInfo`] then its `contract_addr` field converts to a byte string.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            AssetInfo::Native(denom) => denom.as_bytes(),
            AssetInfo::Token(contract_addr) => contract_addr.as_bytes(),
        }
    }
}

impl fmt::Display for AssetInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AssetInfo::Native(denom) => write!(f, "{}", denom),
            AssetInfo::Token(contract_addr) => write!(f, "{}", contract_addr),
        }
    }
}

/// This enum describes available Token types.
/// ## Examples
/// ```
/// # use cosmwasm_std::Addr;
/// # use wyndex::asset::AssetInfo::{Native, Token};
/// Token("terra...".to_string());
/// Native(String::from("uluna"));
/// ```
#[cw_serde]
#[derive(Hash, Eq)]
pub enum AssetInfoValidated {
    /// Non-native Token
    Token(Addr),
    /// Native token
    Native(String),
}

impl From<AssetInfoValidated> for AssetInfo {
    fn from(a: AssetInfoValidated) -> Self {
        match a {
            AssetInfoValidated::Token(addr) => AssetInfo::Token(addr.to_string()),
            AssetInfoValidated::Native(denom) => AssetInfo::Native(denom),
        }
    }
}

impl fmt::Display for AssetInfoValidated {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AssetInfoValidated::Native(denom) => write!(f, "{}", denom),
            AssetInfoValidated::Token(contract_addr) => write!(f, "{}", contract_addr),
        }
    }
}

impl AssetInfoValidated {
    /// Returns true if the caller is a native token. Otherwise returns false.
    pub fn is_native_token(&self) -> bool {
        matches!(self, AssetInfoValidated::Native(_))
    }

    /// Returns `Some(denom)` if this is a native token, or `None` if it is a cw20 token.
    pub fn native_denom(&self) -> Option<&str> {
        match self {
            AssetInfoValidated::Native(denom) => Some(denom),
            _ => None,
        }
    }

    /// Returns the balance of token for the given address.
    ///
    /// * **account_addr** is the address whose token balance we check.
    pub fn query_balance(
        &self,
        querier: &QuerierWrapper,
        account_addr: impl Into<String>,
    ) -> StdResult<Uint128> {
        match self {
            AssetInfoValidated::Token(contract_addr) => {
                query_token_balance(querier, contract_addr, account_addr)
            }
            AssetInfoValidated::Native(denom) => query_balance(querier, account_addr, denom),
        }
    }

    /// Returns the number of decimals that a token has.
    pub fn decimals(&self, querier: &QuerierWrapper) -> StdResult<u8> {
        let decimals = match &self {
            AssetInfoValidated::Native { .. } => NATIVE_TOKEN_PRECISION,
            AssetInfoValidated::Token(contract_addr) => {
                let res: TokenInfoResponse =
                    querier.query_wasm_smart(contract_addr, &Cw20QueryMsg::TokenInfo {})?;

                res.decimals
            }
        };

        Ok(decimals)
    }

    /// Returns **true** if the calling token is the same as the token specified in the input parameters.
    /// Otherwise returns **false**.
    pub fn equal(&self, asset: &AssetInfoValidated) -> bool {
        match (self, asset) {
            (AssetInfoValidated::Native(denom), AssetInfoValidated::Native(other_denom)) => {
                denom == other_denom
            }
            (
                AssetInfoValidated::Token(contract_addr),
                AssetInfoValidated::Token(other_contract_addr),
            ) => contract_addr == other_contract_addr,
            _ => false,
        }
    }

    /// If the caller object is a native token of type [`AssetInfo`] then his `denom` field converts to a byte string.
    ///
    /// If the caller object is a token of type [`AssetInfo`] then its `contract_addr` field converts to a byte string.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            AssetInfoValidated::Native(denom) => denom.as_bytes(),
            AssetInfoValidated::Token(contract_addr) => contract_addr.as_bytes(),
        }
    }
}

impl KeyDeserialize for &AssetInfoValidated {
    type Output = AssetInfoValidated;

    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        let (asset_type, denom) = <(u8, &str)>::from_vec(value)?;

        match asset_type {
            0 => Ok(AssetInfoValidated::Native(denom)),
            1 => Ok(AssetInfoValidated::Token(Addr::unchecked(denom))),
            _ => Err(StdError::generic_err(
                "Invalid AssetInfoValidated key, invalid type",
            )),
        }
    }
}

impl<'a> Prefixer<'a> for &AssetInfoValidated {
    fn prefix(&self) -> Vec<Key> {
        self.key()
    }
}

// Allow using `AssetInfoValidated` as a key in a `Map`
impl<'a> PrimaryKey<'a> for &AssetInfoValidated {
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = Self;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        match self {
            AssetInfoValidated::Native(denom) => {
                vec![Key::Val8([0]), Key::Ref(denom.as_bytes())]
            }
            AssetInfoValidated::Token(addr) => vec![Key::Val8([1]), Key::Ref(addr.as_bytes())],
        }
    }
}

/// Returns a lowercased, validated address upon success if present.
pub fn addr_opt_validate(api: &dyn Api, addr: &Option<String>) -> StdResult<Option<Addr>> {
    addr.as_ref()
        .map(|addr| api.addr_validate(addr))
        .transpose()
}

const TOKEN_SYMBOL_MAX_LENGTH: usize = 4;

/// Returns a formatted LP token name
pub fn format_lp_token_name(
    asset_infos: &[AssetInfoValidated],
    querier: &QuerierWrapper,
) -> StdResult<String> {
    let mut short_symbols: Vec<String> = vec![];
    for asset_info in asset_infos {
        let short_symbol = match &asset_info {
            AssetInfoValidated::Native(denom) => {
                denom.chars().take(TOKEN_SYMBOL_MAX_LENGTH).collect()
            }
            AssetInfoValidated::Token(contract_addr) => {
                let token_symbol = query_token_symbol(querier, contract_addr)?;
                token_symbol.chars().take(TOKEN_SYMBOL_MAX_LENGTH).collect()
            }
        };
        short_symbols.push(short_symbol);
    }
    Ok(format!("{}-LP", short_symbols.iter().join("-")).to_uppercase())
}

/// Returns an [`Asset`] object representing a native token and an amount of tokens.
///
/// * **denom** native asset denomination.
///
/// * **amount** amount of native assets.
pub fn native_asset(denom: impl Into<String>, amount: impl Into<Uint128>) -> AssetValidated {
    AssetValidated {
        info: AssetInfoValidated::Native(denom.into()),
        amount: amount.into(),
    }
}

/// Returns an [`Asset`] object representing a non-native token and an amount of tokens.
/// ## Params
/// * **contract_addr** iaddress of the token contract.
///
/// * **amount** amount of tokens.
pub fn token_asset(contract_addr: Addr, amount: impl Into<Uint128>) -> AssetValidated {
    AssetValidated {
        info: AssetInfoValidated::Token(contract_addr),
        amount: amount.into(),
    }
}

/// Returns an [`AssetInfo`] object representing the denomination for native asset.
pub fn native_asset_info(denom: &str) -> AssetInfo {
    AssetInfo::Native(denom.to_string())
}

/// Returns an [`AssetInfo`] object representing the address of a token contract.
pub fn token_asset_info(contract_addr: &str) -> AssetInfo {
    AssetInfo::Token(contract_addr.to_string())
}

/// Returns [`PairInfo`] by specified pool address.
///
/// * **pool_addr** address of the pool.
pub fn pair_info_by_pool(querier: &QuerierWrapper, pool: impl Into<String>) -> StdResult<PairInfo> {
    let minter_info: MinterResponse = querier.query_wasm_smart(pool, &Cw20QueryMsg::Minter {})?;

    let pair_info: PairInfo =
        querier.query_wasm_smart(minter_info.minter, &PairQueryMsg::Pair {})?;

    Ok(pair_info)
}

/// Checks swap parameters.
///
/// * **pools** amount of tokens in pools.
///
/// * **swap_amount** amount to swap.
pub fn check_swap_parameters(pools: Vec<Uint128>, swap_amount: Uint128) -> StdResult<()> {
    if pools.iter().any(|pool| pool.is_zero()) {
        return Err(StdError::generic_err("One of the pools is empty"));
    }

    if swap_amount.is_zero() {
        return Err(StdError::generic_err("Swap amount must not be zero"));
    }

    Ok(())
}

/// Trait extension for AssetInfo to produce [`Asset`] objects from [`AssetInfo`].
pub trait AssetInfoExt {
    type Asset;
    fn with_balance(&self, balance: impl Into<Uint128>) -> Self::Asset;
}

impl AssetInfoExt for AssetInfoValidated {
    type Asset = AssetValidated;
    fn with_balance(&self, balance: impl Into<Uint128>) -> Self::Asset {
        AssetValidated {
            info: self.clone(),
            amount: balance.into(),
        }
    }
}
impl AssetInfoExt for AssetInfo {
    type Asset = Asset;
    fn with_balance(&self, balance: impl Into<Uint128>) -> Self::Asset {
        Asset {
            info: self.clone(),
            amount: balance.into(),
        }
    }
}

/// Trait extension for Decimal256 to work with token precisions more accurately.
pub trait Decimal256Ext {
    fn to_uint256(&self) -> Uint256;

    fn to_uint128_with_precision(&self, precision: impl Into<u32>) -> StdResult<Uint128>;

    fn to_uint256_with_precision(&self, precision: impl Into<u32>) -> StdResult<Uint256>;

    fn from_integer(i: impl Into<Uint256>) -> Self;

    fn checked_multiply_ratio(
        &self,
        numerator: Decimal256,
        denominator: Decimal256,
    ) -> StdResult<Decimal256>;

    fn with_precision(
        value: impl Into<Uint256>,
        precision: impl Into<u32>,
    ) -> StdResult<Decimal256>;
}

impl Decimal256Ext for Decimal256 {
    fn to_uint256(&self) -> Uint256 {
        self.numerator() / self.denominator()
    }

    fn to_uint128_with_precision(&self, precision: impl Into<u32>) -> StdResult<Uint128> {
        let value = self.atomics();
        let precision = precision.into();

        value
            .checked_div(10u128.pow(self.decimal_places() - precision).into())?
            .try_into()
            .map_err(|o: ConversionOverflowError| {
                StdError::generic_err(format!("Error converting {}", o.value))
            })
    }

    fn to_uint256_with_precision(&self, precision: impl Into<u32>) -> StdResult<Uint256> {
        let value = self.atomics();
        let precision = precision.into();

        value
            .checked_div(10u128.pow(self.decimal_places() - precision).into())
            .map_err(|_| StdError::generic_err("DivideByZeroError"))
    }

    fn from_integer(i: impl Into<Uint256>) -> Self {
        Decimal256::from_ratio(i.into(), 1u8)
    }

    fn checked_multiply_ratio(
        &self,
        numerator: Decimal256,
        denominator: Decimal256,
    ) -> StdResult<Decimal256> {
        Ok(Decimal256::new(
            self.atomics()
                .checked_multiply_ratio(numerator.atomics(), denominator.atomics())
                .map_err(|_| StdError::generic_err("CheckedMultiplyRatioError"))?,
        ))
    }

    fn with_precision(
        value: impl Into<Uint256>,
        precision: impl Into<u32>,
    ) -> StdResult<Decimal256> {
        Decimal256::from_atomics(value, precision.into())
            .map_err(|_| StdError::generic_err("Decimal256 range exceeded"))
    }
}
