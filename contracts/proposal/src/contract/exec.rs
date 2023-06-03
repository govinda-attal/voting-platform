use common::keys::VOTE_DENOM;
use cosmwasm_std::{
    coins, ensure, to_binary, BankMsg, DepsMut, Env, MessageInfo, Response, WasmMsg,
};
use cw_utils::must_pay;

use common::msg::membership::ExecMsg as MembershipExecMsg;
use common::msg::membership::{IsMemberResp, QueryMsg::IsMember};
use distribution::msg::ExecMsg as DistributionExecMsg;

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
    must_pay(&info, VOTE_DENOM)?;

    let sender = info.sender;
    let config = CONFIG.load(deps.storage)?;
    let owner = OWNER.load(deps.storage)?;

    let is_member_resp: IsMemberResp = deps.querier.query_wasm_smart(
        config.membership_contract,
        &IsMember {
            addr: sender.to_string(),
        },
    )?;

    ensure!(is_member_resp.is_member, ContractError::VoteRejected);

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

    let mem_msg = MembershipExecMsg::NewMember {};
    let mem_msg = WasmMsg::Execute {
        contract_addr: config.membership_contract.into_string(),
        msg: to_binary(&mem_msg)?,
        funds: vec![],
    };
    let dis_msg = DistributionExecMsg::DistributeJoiningFee {};
    let dis_msg = WasmMsg::Execute {
        contract_addr: config.distribution_contract.into_string(),
        msg: to_binary(&dis_msg)?,
        funds: coins(fee_paid.u128(), config.joining_fee.denom),
    };

    let resp = Response::new()
        .add_message(mem_msg)
        .add_message(dis_msg)
        .add_attribute("action", "join")
        .add_attribute("sender", sender.as_str())
        .add_attribute("owner", owner.into_string());

    Ok(resp)
}
