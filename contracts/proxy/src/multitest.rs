use anyhow::{Ok, Result as AnyResult};
use common::msg::{ProposalMemberData, ProxyMemberData};
use cosmwasm_std::{from_binary, Addr, Coin, Decimal};
use cw_multi_test::{App, ContractWrapper, Executor};
use cw_utils::parse_execute_response_data;

use crate::msg::{ExecMsg, InstantiateMsg};
use crate::{execute, instantiate, query, reply};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct CodeId(u64);

impl CodeId {
    pub fn store_code(app: &mut App) -> Self {
        let contract = ContractWrapper::new(execute, instantiate, query).with_reply(reply);
        CodeId(app.store_code(Box::new(contract)))
    }

    // #[allow(clippy::too_many_arguments)]
    // #[track_caller]
    // pub fn instantiate(
    //     self,
    //     app: &mut App,
    //     sender: &str,
    //     owner: &str,
    //     distribution_contract: &str,
    //     membership_contract: &str,
    //     label: &str,
    // ) -> AnyResult<Contract> {
    //     Contract::instantiate(
    //         app,
    //         self,
    //         sender,
    //         owner,
    //         distribution_contract,
    //         membership_contract,
    //         label,
    //     )
    // }
}

impl From<CodeId> for u64 {
    fn from(value: CodeId) -> Self {
        value.0
    }
}

#[derive(Debug)]
pub struct Contract(Addr);

impl Contract {
    pub fn from_addr(addr: Addr) -> Self {
        Self(addr)
    }

    pub fn addr(&self) -> &Addr {
        &self.0
    }

    // #[allow(clippy::too_many_arguments)]
    // #[track_caller]
    // pub fn instantiate(
    //     app: &mut App,
    //     code_id: CodeId,
    //     sender: &str,
    //     owner: &str,
    //     distribution_contract: &str,
    //     membership_contract: &str,
    //     label: &str,
    // ) -> AnyResult<Self> {
    //     let ins_msg = InstantiateMsg {
    //         distribution_contract: distribution_contract.to_owned(),
    //         membership_contract: membership_contract.to_owned(),
    //         owner: owner.to_owned(),
    //     };

    //     app.instantiate_contract(
    //         code_id.0,
    //         Addr::unchecked(sender),
    //         &ins_msg,
    //         &[],
    //         label,
    //         None,
    //     )
    //     .map(Self)
    // }

    #[track_caller]
    pub fn propose_member(
        &self,
        app: &mut App,
        sender: &str,
        funds: &[Coin],
        candidate: &str,
    ) -> AnyResult<Option<ProposalMemberData>> {
        let msg = ExecMsg::ProposeMember {
            addr: candidate.to_string(),
        };
        let resp =
            app.execute_contract(Addr::unchecked(sender), self.addr().clone(), &msg, funds)?;
        resp.data
            .map(|data| parse_execute_response_data(&data))
            .transpose()?
            .and_then(|data| data.data)
            .map(|data| from_binary(&data))
            .transpose()
            .map_err(Into::into)
    }

    #[track_caller]
    pub fn buy_vote_tokens<'a>(&self, app: &mut App, sender: &str) -> AnyResult<()> {
        let msg = ExecMsg::BuyVoteTokens {};
        app.execute_contract(Addr::unchecked(sender), self.addr().clone(), &msg, &[])?;
        Ok(())
    }
}
