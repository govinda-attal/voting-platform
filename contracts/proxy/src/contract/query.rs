use common::msg::WithdrawableResp;
use cosmwasm_std::{Deps, Env, StdResult};
use distribution::msg::QueryMsg as DistributionQueryMsg;

use crate::state::CONFIG;

pub fn withdrawable(deps: Deps, env: Env) -> StdResult<WithdrawableResp> {
    let config = CONFIG.load(deps.storage)?;

    let resp: WithdrawableResp = deps.querier.query_wasm_smart(
        config.distribution_contract,
        &DistributionQueryMsg::Withdrawable {
            proxy: env.contract.address.to_string(),
        },
    )?;

    Ok(resp)
}
