use common::keys::VOTE_DENOM;
use cosmwasm_std::{coin, Addr, DepsMut, Env, MessageInfo, Reply, Response};
use cw2::set_contract_version;
use cw_utils::must_pay;

use crate::{
    error::ContractError,
    msg::ExecMsg,
    msg::InstantiateMsg,
    state::{Config, CONFIG, OWNER, VOTER_TOKENS, IS_PASSED},
};

mod exec;
mod reply;

const MEMBER_JOINED_REPLY_ID: u64 = 1;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let proposer = deps.api.addr_validate(&msg.proposer)?;
    let owner = deps.api.addr_validate(&msg.proposed_owner)?;
    let vote_amount = must_pay(&info, VOTE_DENOM)?;

    // addresses are trusted as they come from membership contract
    let distribution_contract = Addr::unchecked(msg.distribution_contract);
    let membership_contract = Addr::unchecked(&msg.membership_contract);

    OWNER.save(deps.storage, &owner)?;

    CONFIG.save(
        deps.storage,
        &Config {
            distribution_contract,
            membership_contract,
            joining_fee: msg.joining_fee,
        },
    )?;

    VOTER_TOKENS.save(
        deps.storage,
        &proposer,
        &coin(vote_amount.u128(), VOTE_DENOM),
    )?;

    IS_PASSED.save(deps.storage, &false)?;

    let resp = Response::new()
        .add_attribute("action", "new_proposal")
        .add_attribute("sender", info.sender.as_str())
        .add_attribute("owner", owner.as_str());
    Ok(resp)
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecMsg,
) -> Result<Response, ContractError> {
    use ExecMsg::*;

    match msg {
        Pass {} => exec::pass(deps, env, info),
        Vote {} => exec::vote(deps, env, info),
        Join {} => exec::join(deps, env, info),
    }
}

pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        MEMBER_JOINED_REPLY_ID => reply::member_joined(reply.result.into_result()),
        id => Err(ContractError::UnrecognizedReplyId(id)),
    }
}
