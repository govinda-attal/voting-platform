use common::keys::VOTE_DENOM;
use common::msg::membership::ExecMsg as MembershipExecMsg;
use cosmwasm_std::{
    coins, ensure, to_binary, DepsMut, Env, MessageInfo, Response, StdResult, SubMsg, Uint128,
    WasmMsg,
};
use cw_utils::must_pay;
use distribution::msg::ExecMsg as DistribtionExecMsg;

use crate::contract::{BUY_VOTE_TOKENS_REPLY_ID, PROPOSE_MEMBER_REPLY_ID, WITHDRAW_REPLY_ID};
use crate::error::ContractError;
use crate::state::{CONFIG, OWNER};

pub fn propose_member(
    deps: DepsMut,
    info: MessageInfo,
    addr: String,
) -> Result<Response, ContractError> {
    let vote_tokens = must_pay(&info, VOTE_DENOM)?;

    let owner = OWNER.load(deps.storage)?;
    ensure!(owner == info.sender, ContractError::Unauthorized);

    let config = CONFIG.load(deps.storage)?;

    let propose_msg = MembershipExecMsg::ProposeMember { addr: addr.clone() };
    let propose_msg = WasmMsg::Execute {
        contract_addr: config.membership_contract.into_string(),
        msg: to_binary(&propose_msg)?,
        funds: coins(vote_tokens.u128(), VOTE_DENOM),
    };

    let propose_msg = SubMsg::reply_on_success(propose_msg, PROPOSE_MEMBER_REPLY_ID);

    let resp = Response::new()
        .add_submessage(propose_msg)
        .add_attribute("action", "propose member")
        .add_attribute("sender", info.sender.as_str())
        .add_attribute("member", addr);

    Ok(resp)
}

pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let owner = OWNER.load(deps.storage)?;
    ensure!(owner == info.sender, ContractError::Unauthorized);

    let config = CONFIG.load(deps.storage)?;

    let withdraw_msg = DistribtionExecMsg::Withdraw {};
    let withdraw_msg = WasmMsg::Execute {
        contract_addr: config.distribution_contract.into_string(),
        msg: to_binary(&withdraw_msg)?,
        funds: vec![],
    };
    let withdraw_msg = SubMsg::reply_on_success(withdraw_msg, WITHDRAW_REPLY_ID);

    let resp = Response::new()
        .add_submessage(withdraw_msg)
        .add_attribute("action", "withdraw")
        .add_attribute("sender", info.sender.as_str());

    Ok(resp)
}

pub fn buy_vote_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let owner = OWNER.load(deps.storage)?;
    ensure!(owner == info.sender, ContractError::Unauthorized);

    let config = CONFIG.load(deps.storage)?;

    let msg = DistribtionExecMsg::BuyVoteTokens {};
    let msg = WasmMsg::Execute {
        contract_addr: config.distribution_contract.into_string(),
        msg: to_binary(&msg)?,
        funds: vec![],
    };
    let msg = SubMsg::reply_on_success(msg, BUY_VOTE_TOKENS_REPLY_ID);

    let resp = Response::new()
        .add_submessage(msg)
        .add_attribute("action", "buy_vote_tokens")
        .add_attribute("sender", info.sender.as_str());

    Ok(resp)
}
