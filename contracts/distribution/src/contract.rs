use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::{
    error::ContractError,
    msg::{ExecMsg, InstantiateMsg, QueryMsg},
    state::{Config, Correction, CONFIG, CORRECTION},
};

mod exec;
mod query;

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

    CONFIG.save(
        deps.storage,
        &Config {
            membership_contract: info.sender,
            new_member_vote_tokens: msg.new_member_vote_tokens,
            vote_token_price: msg.vote_token_price,
        },
    )?;

    CORRECTION.save(deps.storage, &Correction::default())?;

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
        BuyVoteTokens {} => exec::buy_vote_tokens(deps, env, info),
        Withdraw {} => exec::withdraw(deps, env, info),
    }
}

pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        Withdrawable { proxy } => to_binary(&query::withdrawable(deps, env, proxy)?),
    }
}
