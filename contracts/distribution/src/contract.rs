use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use crate::{
    error::ContractError,
    msg::{ExecMsg, InstantiateMsg},
    state::{MEMBERSHIP, NEW_MEMBER_VOTE_TOKENS, VOTE_TOKEN_PRICE},
};

mod exec;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const POINTS_SCALE: u128 = 100;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    MEMBERSHIP.save(deps.storage, &info.sender)?;
    NEW_MEMBER_VOTE_TOKENS.save(deps.storage, &msg.new_member_vote_tokens)?;
    VOTE_TOKEN_PRICE.save(deps.storage, &msg.vote_token_price)?;
    Ok(Response::new().set_data(msg.data))
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecMsg,
) -> Result<Response, ContractError> {
    use ExecMsg::*;
    match msg {
        DistributeJoiningFee { voter_tokens } => {
            exec::distribute_joining_fee(deps, env, info, voter_tokens)
        }
        BuyVoteTokens {} => todo!(),
    }
}
