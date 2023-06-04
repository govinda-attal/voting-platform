use std::collections::HashMap;

use common::keys::{ATOM, VOTE_DENOM};
use cosmwasm_std::{coins, BankMsg, Coin, DepsMut, Env, MessageInfo, Response, Uint128};
use cw_utils::must_pay;

use crate::{
    error::ContractError,
    state::{CORRECTION, MEMBER_CORRECTION},
};

use super::POINTS_SCALE;

pub fn distribute_joining_fee(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    voter_tokens: HashMap<String, Coin>,
) -> Result<Response, ContractError> {
    let total_weight = must_pay(&info, VOTE_DENOM)?.u128();
    let fee_to_distribute = must_pay(&info, ATOM)?.u128();

    let total_points = fee_to_distribute * POINTS_SCALE;
    let ppw = total_points / total_weight;

    let points_to_distribute = ppw * total_weight;

    let mut correction = CORRECTION.may_load(deps.storage)?.unwrap_or_default();
    correction.points_balance += Uint128::new(total_points - points_to_distribute);
    CORRECTION.save(deps.storage, &correction)?;

    let msgs: Vec<_> = voter_tokens
        .into_iter()
        .map(|(addr, votes)| -> Result<_, ContractError> {
            let addr = deps.api.addr_validate(&addr)?;
            let mut correction = MEMBER_CORRECTION.load(deps.storage, &addr)?;
            let weight = votes.amount.u128();
            let points = weight * ppw;
            let amount = points / POINTS_SCALE;
            correction.points_balance += Uint128::new(points % POINTS_SCALE);
            MEMBER_CORRECTION.save(deps.storage, &addr, &correction)?;
            let bank_msg = BankMsg::Send {
                to_address: addr.into_string(),
                amount: coins(amount, ATOM),
            };
            Ok(bank_msg)
        })
        .collect::<Result<_, _>>()?;

    let resp = Response::new()
        .add_messages(msgs)
        .add_attribute("action", "distribute_joining_fee")
        .add_attribute("sender", info.sender.as_str());

    Ok(resp)
}
