use std::collections::HashMap;

use common::keys::VOTE_DENOM;
use cosmwasm_std::{
    coin, coins, ensure, to_binary, Addr, BankMsg, Coin, DepsMut, Empty, Env, MessageInfo, Order,
    Response, SubMsg, Uint128, WasmMsg,
};

use cw_utils::must_pay;
use proposal::msg::{ExecMsg as ProposalExecMsg, InstantiateMsg as ProposalInstantiateMsg};
use proxy::msg::InstantiateMsg as ProxyInstantiateMsg;

use crate::{
    contract::{PROPOSAL_INSTANTIATION_REPLY_ID, PROPOSAL_PASS_REPLY_ID},
    error::ContractError,
    state::{candidates, members, CONFIG},
};

pub fn propose_member(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    addr: String,
) -> Result<Response, ContractError> {
    let addr = deps.api.addr_validate(&addr)?;

    let vote_tokens = must_pay(&info, VOTE_DENOM)?;

    ensure!(
        members().has(deps.storage, &info.sender),
        ContractError::Unauthorized
    );

    ensure!(
        members()
            .idx
            .owner
            .item(deps.storage, addr.clone())?
            .is_none(),
        ContractError::AlreadyAMember
    );

    ensure!(
        !candidates().has(deps.storage, &addr),
        ContractError::ExistingProposalInProgress
    );

    let membership_contract = env.contract.address.into_string();
    let config = CONFIG.load(deps.storage)?;

    let inst_msg = ProposalInstantiateMsg {
        proposer: info.sender.to_string(),
        proposed_owner: addr.to_string(),
        distribution_contract: config.distribution_contract.into_string(),
        membership_contract: membership_contract.clone(),
        joining_fee: config.joining_fee,
    };
    let inst_msg = WasmMsg::Instantiate {
        admin: Some(membership_contract),
        code_id: config.proposal_code_id,
        msg: to_binary(&inst_msg)?,
        funds: coins(vote_tokens.u128(), VOTE_DENOM),
        label: format!("{} Proposal", addr),
    };
    let inst_msg = SubMsg::reply_on_success(inst_msg, PROPOSAL_INSTANTIATION_REPLY_ID);

    let resp = Response::new()
        .add_submessage(inst_msg)
        .add_attribute("action", "propose_member")
        .add_attribute("sender", info.sender.as_str())
        .add_attribute("addr", addr.as_str());
    Ok(resp)
}

pub fn vote_member_proposal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    voter: String,
    voter_proxy: String,
) -> Result<Response, ContractError> {
    let voter = deps.api.addr_validate(&voter)?;
    let voter_proxy = deps.api.addr_validate(&voter_proxy)?;
    ensure!(
        members().has(deps.storage, &info.sender),
        ContractError::Unauthorized
    );
    ensure!(
        !members().has(deps.storage, &info.sender),
        ContractError::AlreadyAMember
    );

    ensure!(
        candidates()
            .idx
            .proposal
            .item(deps.storage, info.sender.clone())?
            .is_some(),
        ContractError::NotProposedMember
    );

    ensure!(
        members().load(deps.storage, &voter_proxy)? == voter,
        ContractError::MemberProxyMismatch
    );

    // let owner: Addr = proposal::state::OWNER.query(&deps.querier, info.sender)?;

    let vote_tokens = deps
        .querier
        .query_balance(info.sender.clone(), VOTE_DENOM)?;

    let config = CONFIG.load(deps.storage)?;

    let total_vote_tokens_in_circulation = distribution::state::TOTAL_VOTE_TOKENS_IN_CIRCULATION
        .query(&deps.querier, config.distribution_contract)?;

    let mut resp = Response::new()
        .add_attribute("action", "vote_member_proposal")
        .add_attribute("sender", info.sender.as_str());

    if vote_tokens.amount < total_vote_tokens_in_circulation.amount / Uint128::new(2) {
        resp = resp.add_attribute("passed", "no");
        return Ok(resp);
    }
    resp = resp.add_attribute("passed", "yes");

    let msg = ProposalExecMsg::Pass {};
    let msg = WasmMsg::Execute {
        contract_addr: info.sender.into_string(),
        msg: to_binary(&msg)?,
        funds: vec![],
    };

    let msg = SubMsg::reply_on_success(msg, PROPOSAL_PASS_REPLY_ID);

    Ok(resp.add_submessage(msg))
}

pub fn new_member(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    ensure!(
        !members().has(deps.storage, &info.sender),
        ContractError::AlreadyAMember
    );

    let new_member_vote_amount = must_pay(&info, VOTE_DENOM)?;

    let proposal_addr = info.sender;
    let proposal_owner = proposal::state::OWNER.query(&deps.querier, proposal_addr.clone())?;

    ensure!(
        candidates().has(deps.storage, &proposal_owner),
        ContractError::NotProposedMember
    );

    candidates().remove(deps.storage, &proposal_owner)?;

    let config = CONFIG.load(deps.storage)?;

    let membership_contract = env.contract.address.into_string();
    let msg = ProxyInstantiateMsg {
        owner: proposal_owner.clone().into_string(),
        distribution_contract: config.distribution_contract.to_string(),
        membership_contract: membership_contract.clone(),
    };

    let msg = WasmMsg::Instantiate {
        admin: Some(membership_contract),
        code_id: config.proxy_code_id,
        msg: to_binary(&msg)?,
        funds: coins(new_member_vote_amount.u128(), VOTE_DENOM),
        label: format!("{} Proxy", proposal_owner),
    };

    let msg = SubMsg::reply_on_success(msg, super::PROXY_INSTANTIATION_REPLY_ID);

    let resp = Response::new()
        .add_submessage(msg)
        .add_attribute("action", "new_member")
        .add_attribute("sender", proposal_addr.as_str())
        .add_attribute("owner", proposal_owner.as_str());

    Ok(resp)
}
