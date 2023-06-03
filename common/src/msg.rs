use cosmwasm_schema::cw_serde;

pub mod membership {
    use cosmwasm_schema::QueryResponses;

    use super::*;

    #[cw_serde]
    pub enum ExecMsg {
        ProposeMember { addr: String },
        VoteMemberProposal {},
        NewMember {},
    }

    #[cw_serde]
    #[derive(QueryResponses)]
    pub enum QueryMsg {
        #[returns(IsMemberResp)]
        IsMember { addr: String },
    }

    #[cw_serde]
    pub struct IsMemberResp {
        pub is_member: bool,
    }
}

#[cw_serde]
pub struct ProposalMemberData {
    pub owner_addr: String,
    pub proposal_addr: String,
}

#[cw_serde]
pub struct ProxyMemberData {
    pub owner_addr: String,
    pub proxy_addr: String,
}
