use common::msg::WithdrawableResp;
use cosmwasm_std::{Addr, Deps, Env, StdResult, Uint128};

use crate::state::MEMBER_DATA;

use super::POINTS_SCALE;

pub fn withdrawable(deps: Deps, _env: Env, proxy: String) -> StdResult<WithdrawableResp> {
    let proxy = Addr::unchecked(proxy);

    let member_data = MEMBER_DATA
        .may_load(deps.storage, &proxy)?
        .unwrap_or_default();

    let mut reward_funds = member_data.reward_balance.clone();
    reward_funds.amount += member_data.points_balance / Uint128::new(POINTS_SCALE);

    if reward_funds.amount.is_zero() {
        return Ok(WithdrawableResp::default());
    }
    Ok(WithdrawableResp {
        funds: Some(reward_funds),
    })
}
