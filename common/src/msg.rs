use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;

pub mod membership {
    use cosmwasm_schema::QueryResponses;

    use super::*;

    #[cw_serde]
    pub enum ExecMsg {
        ProposeMember { addr: String },
        VoteMemberProposal { voter: String, voter_proxy: String },
        NewMember {},
    }

    #[cw_serde]
    #[derive(QueryResponses)]
    pub enum QueryMsg {
        #[returns(IsMemberResp)]
        IsMember { addr: String },
        #[returns(IsMemberResp)]
        IsProposedMember { addr: String },
        #[returns(OwnerProxyResp)]
        OwnerProxy { owner: String },
    }

    #[cw_serde]
    pub struct IsMemberResp {
        pub ok: bool,
    }

    #[cw_serde]
    pub struct IsProposedMemberResp {
        pub ok: bool,
    }

    #[cw_serde]
    pub struct OwnerProxyResp {
        pub owner: String,
        pub proxy: String,
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

#[cw_serde]
#[derive(Default)]
pub struct WithdrawableResp {
    pub funds: Option<Coin>,
}
