use common::keys::{ATOM, VOTE_DENOM};
use cosmwasm_std::{
    coin, ensure, to_binary, Addr, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::must_pay;

use crate::{
    error::ContractError,
    msg::InstantiateMsg,
    state::{Config, CONFIG},
};
use common::msg::membership::{ExecMsg, QueryMsg};
use distribution::msg::InstantiateMsg as DistributionInstantiateMsg;

mod exec;
mod query;
mod reply;

const INITIAL_PROXY_INSTANTIATION_REPLY_ID: u64 = 1;
const DISTRIBUTION_INSTANTIATION_REPLY_ID: u64 = 2;
const PROPOSAL_INSTANTIATION_REPLY_ID: u64 = 3;
const PROPOSAL_PASS_REPLY_ID: u64 = 4;
const PROXY_INSTANTIATION_REPLY_ID: u64 = 5;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    ensure!(
        msg.initial_members.len() >= 2,
        ContractError::NotEnoughInitialMembers
    );

    let vote_funds = must_pay(&info, VOTE_DENOM)?;

    ensure!(
        vote_funds
            >= Uint128::new(msg.initial_members.len() as u128) * msg.new_member_vote_tokens.amount,
        ContractError::InitialisationLessVoteTokens
    );

    ensure!(
        msg.new_member_vote_tokens.amount >= Uint128::new(2),
        ContractError::NotEnoughNewMemberVoteTokens
    );

    ensure!(
        msg.joining_fee.denom == ATOM,
        ContractError::JoiningFeeDenomInvalid {
            denom: ATOM.to_string()
        }
    );

    let config = Config {
        proposal_code_id: msg.proposal_code_id,
        proxy_code_id: msg.proxy_code_id,
        distribution_contract: Addr::unchecked(""), // will get it in reply!
        joining_fee: msg.joining_fee,
    };

    CONFIG.save(deps.storage, &config)?;

    let members_data = to_binary(&msg.initial_members)?;
    let membership_contract = env.contract.address.to_string();

    let instantiate_msg = DistributionInstantiateMsg {
        new_member_vote_tokens: msg.new_member_vote_tokens,
        vote_token_price: msg.vote_token_price,
        total_vote_tokens_in_circulation: coin(vote_funds.u128(), VOTE_DENOM),
        data: members_data,
    };

    let instantiate_msg = WasmMsg::Instantiate {
        admin: Some(membership_contract),
        code_id: msg.distribution_code_id,
        msg: to_binary(&instantiate_msg)?,
        funds: vec![],
        label: "Distribution".to_owned(),
    };

    let instantiate_msg =
        SubMsg::reply_on_success(instantiate_msg, DISTRIBUTION_INSTANTIATION_REPLY_ID);

    let resp = Response::new().add_submessage(instantiate_msg);

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
        // this is called by proxy contract
        ProposeMember { addr } => exec::propose_member(deps, env, info, addr),
        // this is called by proposal contract
        VoteMemberProposal {} => exec::vote_member_proposal(deps, env, info),
        NewMember {} => exec::new_member(deps, env, info),
    }
}

pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        DISTRIBUTION_INSTANTIATION_REPLY_ID => {
            reply::distribution_instantiated(deps, env, reply.result.into_result())
        }
        INITIAL_PROXY_INSTANTIATION_REPLY_ID => {
            reply::initial_proxy_instantiated(deps, reply.result.into_result())
        }
        PROPOSAL_INSTANTIATION_REPLY_ID => {
            reply::proposal_instantiated(deps, reply.result.into_result())
        }
        PROPOSAL_PASS_REPLY_ID => reply::proposal_passed(deps, env, reply.result.into_result()),
        PROXY_INSTANTIATION_REPLY_ID => reply::proxy_instantiated(deps, reply.result.into_result()),
        id => Err(ContractError::UnrecognizedReplyId(id)),
    }
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        IsMember { addr } => query::is_member(deps, addr).and_then(|resp| to_binary(&resp)),
    }
}
