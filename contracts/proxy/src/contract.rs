use common::keys::VOTE_DENOM;
use cosmwasm_std::{
    ensure, to_binary, Addr, BankMsg, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdResult,
};
use cw2::set_contract_version;

mod exec;
mod reply;

use crate::error::ContractError;
use crate::msg::{ExecMsg, InstantiateMsg};
use crate::state::{Config, CONFIG, OWNER};

const PROPOSE_MEMBER_REPLY_ID: u64 = 1;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;

    // addresses are trusted as they come from membership contract
    let distribution_contract = Addr::unchecked(msg.distribution_contract);
    let membership_contract = deps.api.addr_validate(&msg.membership_contract)?;

    OWNER.save(deps.storage, &owner)?;

    CONFIG.save(
        deps.storage,
        &Config {
            distribution_contract,
            membership_contract,
        },
    )?;

    // move vote_tokens to actual owner
    let vote_coins = deps
        .querier
        .query_balance(env.contract.address, VOTE_DENOM)?;
    let bank_msg = BankMsg::Send {
        to_address: owner.into_string(),
        amount: vec![vote_coins],
    };

    Ok(Response::new().add_message(bank_msg))
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecMsg,
) -> Result<Response, ContractError> {
    use ExecMsg::*;

    match msg {
        ProposeMember { addr } => exec::propose_member(deps, info, addr),
        BuyVoteTokens {} => todo!(),
    }
}

pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        PROPOSE_MEMBER_REPLY_ID => reply::propose_member(reply.result.into_result()),
        id => Err(ContractError::UnrecognizedReplyId(id)),
    }
}
