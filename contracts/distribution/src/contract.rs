use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::{
    error::ContractError,
    msg::InstantiateMsg,
    state::{MEMBERSHIP, NEW_MEMBER_VOTE_TOKENS, VOTE_TOKEN_PRICE},
};

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    MEMBERSHIP.save(deps.storage, &info.sender)?;
    NEW_MEMBER_VOTE_TOKENS.save(deps.storage, &msg.new_member_vote_tokens)?;
    VOTE_TOKEN_PRICE.save(deps.storage, &msg.vote_token_price)?;
    Ok(Response::new().set_data(msg.data))
}
