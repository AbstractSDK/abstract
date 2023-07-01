use cosmwasm_std::{to_binary, Addr, Decimal, StdResult, SubMsg, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;

use wynd_curve_utils::{Curve, PiecewiseLinear, SaturatingLinear};

use crate::state::Config;

pub fn create_undelegate_msg(
    recipient: Addr,
    amount: Uint128,
    contract: Addr,
) -> StdResult<SubMsg> {
    let undelegate = Cw20ExecuteMsg::Transfer {
        recipient: recipient.to_string(),
        amount,
    };
    Ok(SubMsg::new(WasmMsg::Execute {
        contract_addr: contract.to_string(),
        msg: to_binary(&undelegate)?,
        funds: vec![],
    }))
}

pub fn calc_power(cfg: &Config, stake: Uint128, multiplier: Decimal) -> Uint128 {
    if stake < cfg.min_bond {
        Uint128::zero()
    } else {
        stake * multiplier / cfg.tokens_per_power
    }
}

pub trait CurveExt {
    /// Shifts this curve to the right by `x` units.
    fn shift(self, x: u64) -> Curve;

    /// Returns the last `x` value of the curve, if any.
    /// This will be `None` for infinite and empty curves.
    fn end(&self) -> Option<u64>;
}

impl CurveExt for Curve {
    fn shift(self, x: u64) -> Curve {
        match self {
            c @ Curve::Constant { .. } => c,
            Curve::SaturatingLinear(sl) => sl.shift(x),
            Curve::PiecewiseLinear(pl) => pl.shift(x),
        }
    }

    fn end(&self) -> Option<u64> {
        match self {
            Curve::Constant { .. } => None,
            Curve::SaturatingLinear(sl) => sl.end(),
            Curve::PiecewiseLinear(pl) => pl.end(),
        }
    }
}

impl CurveExt for SaturatingLinear {
    fn shift(mut self, x: u64) -> Curve {
        self.min_x = self.min_x.checked_add(x).unwrap();
        self.max_x = self.max_x.checked_add(x).unwrap();

        Curve::SaturatingLinear(self)
    }

    fn end(&self) -> Option<u64> {
        Some(self.max_x)
    }
}

impl CurveExt for PiecewiseLinear {
    fn shift(mut self, by: u64) -> Curve {
        for (x, _) in self.steps.iter_mut() {
            *x = x.checked_add(by).unwrap();
        }
        Curve::PiecewiseLinear(self)
    }

    fn end(&self) -> Option<u64> {
        self.steps.last().map(|(x, _)| *x)
    }
}
