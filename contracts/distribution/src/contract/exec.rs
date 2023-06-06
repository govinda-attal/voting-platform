use std::collections::HashMap;

use common::keys::{ATOM, VOTE_DENOM};

use common::msg::membership::{IsMemberResp, QueryMsg as MembershipQueryMsg};

use cosmwasm_std::{
    coin, coins, ensure, BankMsg, Coin, DepsMut, Env, Event, MessageInfo, Response, Uint128,
};

use crate::{
    error::ContractError,
    state::{MemberData, CONFIG, CORRECTION, MEMBER_DATA},
};

use super::POINTS_SCALE;

pub fn distribute_joining_fee(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    voter_tokens: HashMap<String, Coin>,
) -> Result<Response, ContractError> {
    let incoming_vote_tokens = info
        .funds
        .iter()
        .find(|coin| coin.denom == VOTE_DENOM)
        .ok_or(ContractError::ExpectedBalanceVoteTokens)?
        .amount
        .u128();
    let fee_to_distribute = info
        .funds
        .iter()
        .find(|coin| coin.denom == ATOM)
        .ok_or(ContractError::ExpectedJoiningFeeInAtoms)?
        .amount
        .u128();

    let config = CONFIG.load(deps.storage)?;
    // Membership at the time of instantiation of new proxy for new joining member transfers new member tokens directly
    // Membership passed balance of balance vote tokens to distribution
    // this helps to calculate total weight and distribute rewards among voters
    let total_weight = config.new_member_vote_tokens.amount.u128() + incoming_vote_tokens;

    let total_points = fee_to_distribute * POINTS_SCALE;
    let ppw = total_points / total_weight;

    let points_to_distribute = ppw * total_weight;

    let mut correction = CORRECTION.load(deps.storage)?;
    correction.points_balance += Uint128::new(total_points - points_to_distribute);
    CORRECTION.save(deps.storage, &correction)?;

    let events: Vec<_> = voter_tokens
        .into_iter()
        .map(|(addr, votes)| -> Result<_, ContractError> {
            let addr = deps.api.addr_validate(&addr)?;
            let mut data = MEMBER_DATA
                .may_load(deps.storage, &addr)?
                .unwrap_or(MemberData::default().with_reward_balance(coin(0, ATOM)));
            let weight = votes.amount.u128();
            let points = weight * ppw;
            let amount = points / POINTS_SCALE;
            data.points_balance += Uint128::new(points % POINTS_SCALE);
            data.reward_balance.amount += Uint128::new(amount);

            MEMBER_DATA.save(deps.storage, &addr, &data)?;

            let event = Event::new("reward_distribution")
                .add_attribute("voter_proxy", addr.as_str())
                .add_attribute("voter_weight", votes.amount.to_string())
                .add_attribute("reward_amount", amount.to_string())
                .add_attribute("reward_token", ATOM);
            Ok(event)
        })
        .collect::<Result<_, _>>()?;

    let resp = Response::new()
        .add_events(events)
        .add_attribute("action", "distribute_joining_fee")
        .add_attribute("sender", info.sender.as_str());

    Ok(resp)
}

pub fn withdraw(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let is_member: IsMemberResp = deps.querier.query_wasm_smart(
        config.membership_contract,
        &MembershipQueryMsg::IsMember {
            addr: info.sender.to_string(),
        },
    )?;

    ensure!(is_member.ok, ContractError::Unauthorized);

    let mut data = MEMBER_DATA
        .may_load(deps.storage, &info.sender)?
        .unwrap_or(MemberData::default().with_reward_balance(coin(0, ATOM)));

    let reward_amount =
        data.reward_balance.amount + data.points_balance / Uint128::new(POINTS_SCALE);
    data.reward_balance.amount -= reward_amount;
    data.points_balance = data.points_balance % Uint128::new(POINTS_SCALE);

    MEMBER_DATA.save(deps.storage, &info.sender, &data)?;

    let mut resp = Response::new()
        .add_attribute("action", "withdraw")
        .add_attribute("sender", info.sender.as_str());

    if reward_amount.u128() > 0 {
        resp = resp.add_message(BankMsg::Send {
            to_address: info.sender.into(),
            amount: coins(reward_amount.u128(), ATOM),
        });
    }

    Ok(resp)
}

pub fn buy_vote_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let is_member: IsMemberResp = deps.querier.query_wasm_smart(
        config.membership_contract,
        &MembershipQueryMsg::IsMember {
            addr: info.sender.to_string(),
        },
    )?;

    ensure!(is_member.ok, ContractError::Unauthorized);

    let mut data = MEMBER_DATA
        .may_load(deps.storage, &info.sender)?
        .unwrap_or(MemberData::default().with_reward_balance(coin(0, ATOM)));

    let reward_amount =
        data.reward_balance.amount + data.points_balance / Uint128::new(POINTS_SCALE);
    data.reward_balance.amount -= reward_amount;
    data.points_balance = data.points_balance % Uint128::new(POINTS_SCALE);

    let vote_amount = reward_amount / config.vote_token_price.amount;
    data.reward_balance.amount += reward_amount % config.vote_token_price.amount;

    MEMBER_DATA.save(deps.storage, &info.sender, &data)?;

    let mut resp = Response::new()
        .add_attribute("action", "buy_vote_token")
        .add_attribute("sender", info.sender.as_str());

    let available_vote_amount = deps
        .querier
        .query_balance(env.contract.address, VOTE_DENOM)?
        .amount;

    if vote_amount.u128() > 0 && available_vote_amount > vote_amount {
        resp = resp.add_message(BankMsg::Send {
            to_address: info.sender.into(),
            amount: coins(vote_amount.u128(), VOTE_DENOM),
        });
    }

    Ok(resp)
}
