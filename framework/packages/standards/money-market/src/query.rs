use crate::{msg::MoneyMarketQueryMsg, MoneyMarketCommand, MoneyMarketError};
use abstract_sdk::Resolve;
use abstract_std::objects::ans_host::AnsHostError;

/// This is an alias for a resolve_money_market function.
pub type PlatformResolver =
    fn(value: &str) -> Result<Box<dyn MoneyMarketCommand>, MoneyMarketError>;

/// Structure created to be able to resolve an action using ANS
pub struct MoneyMarketQueryResolveWrapper(pub PlatformResolver, pub MoneyMarketQueryMsg);

pub fn err(e: MoneyMarketError) -> AnsHostError {
    AnsHostError::QueryFailed {
        method_name: "resolve money market".to_string(),
        error: cosmwasm_std::StdError::GenericErr { msg: e.to_string() },
    }
}

impl Resolve for MoneyMarketQueryResolveWrapper {
    type Output = MoneyMarketQueryMsg;

    /// TODO: this only works for protocols where there is only one address for depositing
    fn resolve(
        &self,
        querier: &cosmwasm_std::QuerierWrapper,
        ans_host: &abstract_sdk::feature_objects::AnsHost,
    ) -> abstract_std::objects::ans_host::AnsHostResult<Self::Output> {
        let raw_action = match self.1.clone() {
            MoneyMarketQueryMsg::AnsUserDeposit {
                user,
                asset,
                money_market,
            } => {
                let platform = self.0(&money_market).map_err(err)?;
                let contract_addr = platform.lending_address(querier, ans_host, asset.clone())?;
                let asset = asset.resolve(querier, ans_host)?;
                MoneyMarketQueryMsg::RawUserDeposit {
                    asset: asset.into(),
                    user,
                    contract_addr: contract_addr.to_string(),
                    money_market,
                }
            }
            MoneyMarketQueryMsg::AnsUserCollateral {
                user,
                collateral_asset,
                borrowed_asset,
                money_market,
            } => {
                let platform = self.0(&money_market).map_err(err)?;
                let contract_addr = platform.collateral_address(
                    querier,
                    ans_host,
                    borrowed_asset.clone(),
                    collateral_asset.clone(),
                )?;
                let collateral_asset = collateral_asset.resolve(querier, ans_host)?;
                let borrowed_asset = borrowed_asset.resolve(querier, ans_host)?;
                MoneyMarketQueryMsg::RawUserCollateral {
                    user,
                    collateral_asset: collateral_asset.into(),
                    borrowed_asset: borrowed_asset.into(),
                    contract_addr: contract_addr.to_string(),
                    money_market,
                }
            }
            MoneyMarketQueryMsg::AnsUserBorrow {
                user,
                collateral_asset,
                borrowed_asset,
                money_market,
            } => {
                let platform = self.0(&money_market).map_err(err)?;
                let contract_addr = platform.borrow_address(
                    querier,
                    ans_host,
                    borrowed_asset.clone(),
                    collateral_asset.clone(),
                )?;
                let collateral_asset = collateral_asset.resolve(querier, ans_host)?;
                let borrowed_asset = borrowed_asset.resolve(querier, ans_host)?;
                MoneyMarketQueryMsg::RawUserBorrow {
                    user,
                    collateral_asset: collateral_asset.into(),
                    borrowed_asset: borrowed_asset.into(),

                    contract_addr: contract_addr.to_string(),
                    money_market,
                }
            }
            MoneyMarketQueryMsg::AnsCurrentLTV {
                user,
                collateral_asset,
                borrowed_asset,
                money_market,
            } => {
                let platform = self.0(&money_market).map_err(err)?;
                let contract_addr = platform.current_ltv_address(
                    querier,
                    ans_host,
                    borrowed_asset.clone(),
                    collateral_asset.clone(),
                )?;
                let collateral_asset = collateral_asset.resolve(querier, ans_host)?;
                let borrowed_asset = borrowed_asset.resolve(querier, ans_host)?;
                MoneyMarketQueryMsg::RawCurrentLTV {
                    user,
                    collateral_asset: collateral_asset.into(),
                    borrowed_asset: borrowed_asset.into(),

                    contract_addr: contract_addr.to_string(),
                    money_market,
                }
            }
            MoneyMarketQueryMsg::AnsMaxLTV {
                user,
                collateral_asset,
                borrowed_asset,
                money_market,
            } => {
                let platform = self.0(&money_market).map_err(err)?;
                let contract_addr = platform.max_ltv_address(
                    querier,
                    ans_host,
                    borrowed_asset.clone(),
                    collateral_asset.clone(),
                )?;
                let collateral_asset = collateral_asset.resolve(querier, ans_host)?;
                let borrowed_asset = borrowed_asset.resolve(querier, ans_host)?;
                MoneyMarketQueryMsg::RawMaxLTV {
                    user,
                    collateral_asset: collateral_asset.into(),
                    borrowed_asset: borrowed_asset.into(),

                    contract_addr: contract_addr.to_string(),
                    money_market,
                }
            }
            MoneyMarketQueryMsg::AnsPrice {
                quote,
                base,
                money_market,
            } => {
                let quote = quote.resolve(querier, ans_host)?;
                let base = base.resolve(querier, ans_host)?;
                MoneyMarketQueryMsg::RawPrice {
                    quote: quote.into(),
                    base: base.into(),
                    money_market,
                }
            }
            _ => self.1.clone(),
        };

        Ok(raw_action)
    }
}
