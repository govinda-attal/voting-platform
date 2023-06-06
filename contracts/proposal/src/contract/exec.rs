use std::collections::HashMap;

use common::keys::VOTE_DENOM;
use cosmwasm_std::{
    coin, coins, ensure, to_binary, Addr, BankMsg, DepsMut, Env, MessageInfo, Response, StdResult,
    SubMsg, WasmMsg,
};
use cw_utils::must_pay;

use common::msg::membership::{ExecMsg as MembershipExecMsg, IsProposedMemberResp, OwnerProxyResp};
use common::msg::membership::{IsMemberResp, QueryMsg::IsProposedMember, QueryMsg::OwnerProxy};
use distribution::msg::ExecMsg as DistributionExecMsg;

use crate::contract::MEMBER_JOINED_REPLY_ID;
use crate::state::VOTER_TOKENS;
use crate::{
    error::ContractError,
    state::{CONFIG, IS_PASSED, OWNER},
};

pub fn pass(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    ensure!(
        info.sender == config.membership_contract,
        ContractError::Unauthorized
    );

    IS_PASSED.save(deps.storage, &true)?;
    let owner = OWNER.load(deps.storage)?;

    let resp = Response::new()
        .add_attribute("action", "pass_proposal")
        .add_attribute("sender", info.sender.as_str())
        .add_attribute("owner", owner.into_string());

    Ok(resp)
}

pub fn vote(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let vote_amount = must_pay(&info, VOTE_DENOM)?;

    let sender = info.sender;
    let config = CONFIG.load(deps.storage)?;
    let is_passed = IS_PASSED.load(deps.storage)?;
    let owner = OWNER.load(deps.storage)?;

   
    ensure!(!is_passed, ContractError::VoteRejectedProposalWasPassedEarlier);

    let sender_proxy_resp: OwnerProxyResp = deps.querier.query_wasm_smart(
        config.membership_contract,
        &OwnerProxy {
            owner: sender.to_string(),
        },
    )?;

    let sender_proxy = Addr::unchecked(sender_proxy_resp.proxy);

    VOTER_TOKENS.update(deps.storage, &sender_proxy, |votes| -> StdResult<_> {
        let votes = votes.map_or_else(
            || coin(vote_amount.u128(), VOTE_DENOM),
            |c| coin((c.amount + vote_amount).u128(), c.denom),
        );
        Ok(votes)
    })?;
    let config = CONFIG.load(deps.storage)?;

    let mem_msg = MembershipExecMsg::VoteMemberProposal {
        voter: owner.to_string(),
        voter_proxy: sender_proxy.to_string(),
    };
    let mem_msg = WasmMsg::Execute {
        contract_addr: config.membership_contract.into_string(),
        msg: to_binary(&mem_msg)?,
        funds: vec![],
    };

    let resp = Response::new()
        .add_attribute("action", "vote_member_proposal")
        .add_attribute("sender", sender.as_str())
        .add_attribute("owner", owner.into_string());

    Ok(resp)
}

pub fn join(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let fee_paid = must_pay(&info, &config.joining_fee.denom)?;

    ensure!(
        fee_paid >= config.joining_fee.amount,
        ContractError::JoinRejected {
            fee: config.joining_fee
        }
    );

    let sender = info.sender;
    let owner = OWNER.load(deps.storage)?;

    ensure!(sender == owner, ContractError::Unauthorized);

    let is_proposed_member: IsProposedMemberResp = deps.querier.query_wasm_smart(
        config.membership_contract.clone(),
        &IsProposedMember {
            addr: owner.to_string(),
        },
    )?;

    ensure!(is_proposed_member.ok, ContractError::Unauthorized);

    let vote_tokens = deps
        .querier
        .query_balance(env.contract.address, VOTE_DENOM)?;

    let mem_msg = MembershipExecMsg::NewMember {};
    let mem_msg = WasmMsg::Execute {
        contract_addr: config.membership_contract.into_string(),
        msg: to_binary(&mem_msg)?,
        funds: vec![vote_tokens.clone()],
    };

    let mem_msg = SubMsg::reply_on_success(mem_msg, MEMBER_JOINED_REPLY_ID);

    let voter_tokens: HashMap<_, _> = VOTER_TOKENS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .into_iter()
        .map(|votes| -> StdResult<_> {
            let votes = votes?;
            Ok((votes.0.to_string(), votes.1))
        })
        .collect::<Result<_, _>>()?;

    let dis_msg = DistributionExecMsg::DistributeJoiningFee {
        total_vote_tokens: vote_tokens,
        voter_tokens,
    };
    let dis_msg = WasmMsg::Execute {
        contract_addr: config.distribution_contract.into_string(),
        msg: to_binary(&dis_msg)?,
        funds: coins(fee_paid.u128(), config.joining_fee.denom),
    };

    let resp = Response::new()
        .add_submessage(mem_msg)
        .add_message(dis_msg)
        .add_attribute("action", "join")
        .add_attribute("sender", sender.as_str())
        .add_attribute("owner", owner.into_string());

    Ok(resp)
}
