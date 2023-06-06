use std::string::ParseError;

use crate::error::ContractError;
use crate::msg::{InstantiateMsg, InstantiationData};
use crate::state::{Config, CONFIG};
use crate::{execute, instantiate, query, reply};
use anyhow::Result as AnyResult;
use common::msg::membership::{ExecMsg, IsMemberResp, QueryMsg};
use common::msg::{ProposalMemberData, ProxyMemberData};
use cosmwasm_std::{from_binary, to_binary, Addr, Coin, Decimal, WasmMsg};
use cw_multi_test::{App, ContractWrapper, Executor};
use cw_utils::{parse_execute_response_data, parse_instantiate_response_data};

use distribution::multitest::CodeId as DistributionId;
use proposal::multitest::{CodeId as ProposalId, Contract as ProposalContract};
use proxy::multitest::{CodeId as ProxyId, Contract as ProxyContract};

#[cfg(test)]
mod tests;

pub struct CodeId(u64);

impl CodeId {
    pub fn store_code(app: &mut App) -> Self {
        let contract = ContractWrapper::new(execute, instantiate, query).with_reply(reply);
        CodeId(app.store_code(Box::new(contract)))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn instantiate(
        self,
        app: &mut App,
        sender: &Addr,
        new_member_vote_tokens: Coin,
        vote_token_price: Coin,
        joining_fee: Coin,
        proxy_code_id: ProxyId,
        proposal_code_id: ProposalId,
        distribution_code_id: DistributionId,
        initial_members: &[&str],
        label: &str,
        funds: &[Coin],
    ) -> AnyResult<(Contract, InstantiationData)> {
        Contract::instantiate(
            app,
            self,
            sender,
            new_member_vote_tokens,
            vote_token_price,
            joining_fee,
            proxy_code_id,
            proposal_code_id,
            distribution_code_id,
            initial_members,
            label,
            funds,
        )
    }
}

#[derive(Debug)]
pub struct Contract(Addr);

impl Contract {
    pub fn addr(&self) -> &Addr {
        &self.0
    }

    #[allow(clippy::too_many_arguments)]
    // #[track_caller]
    pub fn instantiate(
        app: &mut App,
        code_id: CodeId,
        sender: &Addr,
        new_member_vote_tokens: Coin,
        vote_token_price: Coin,
        joining_fee: Coin,
        proxy_code_id: ProxyId,
        proposal_code_id: ProposalId,
        distribution_code_id: DistributionId,
        initial_members: &[&str],
        label: &str,
        funds: &[Coin],
    ) -> AnyResult<(Contract, InstantiationData)> {
        let msg = InstantiateMsg {
            new_member_vote_tokens,
            vote_token_price,
            joining_fee,
            proxy_code_id: proxy_code_id.into(),
            proposal_code_id: proposal_code_id.into(),
            distribution_code_id: distribution_code_id.into(),
            initial_members: initial_members.iter().map(|s| s.to_string()).collect(),
        };

        let msg = WasmMsg::Instantiate {
            admin: Some(sender.to_string()),
            code_id: code_id.0,
            msg: to_binary(&msg)?,
            funds: funds.to_vec(),
            label: label.into(),
        };

        let res = app.execute(sender.clone(), msg.into())?;

        let data = parse_instantiate_response_data(res.data.unwrap_or_default().as_slice())?;

        let contract = Self(Addr::unchecked(data.contract_address));
        let data = from_binary(&data.data.unwrap_or_default())?;
        Ok((contract, data))
    }

    pub fn is_member(&self, app: &App, addr: &str) -> AnyResult<IsMemberResp> {
        let query = QueryMsg::IsMember {
            addr: addr.to_owned(),
        };

        app.wrap()
            .query_wasm_smart(self.0.clone(), &query)
            .map_err(Into::into)
    }

    pub fn load_config(&self, app: &App) -> Config {
        CONFIG.query(&app.wrap(), self.addr().clone()).unwrap()
    }
}
