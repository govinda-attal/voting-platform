use std::collections::HashMap;

use common::keys::{ATOM, VOTE_DENOM};
use cosmwasm_std::{coin, coins, Addr};
use cw_multi_test::App;

use super::CodeId as MembershipId;
use distribution::multitest::CodeId as DistributionId;
use proxy::multitest::{CodeId as ProxyId, Contract as ProxyContract};
use proxy::multitest::{CodeId as ProposalId, Contract as ProposalContract};

#[test]
fn test_member_vote_flow() {
    let mut app = App::default();

    let admin = "admin";
    let members = ["member1", "member2"];
    let candidate = "candidate";

    let proxy_id = ProxyId::store_code(&mut app);
    let proposal_id = ProposalId::store_code(&mut app);
    let distribution_id = DistributionId::store_code(&mut app);
    let membership_id = MembershipId::store_code(&mut app);

    let (membership, data) = membership_id
        .instantiate(
            &mut app,
            admin,
            coin(2, VOTE_DENOM),
            coin(1, ATOM),
            coin(30, ATOM),
            proxy_id,
            proposal_id,
            distribution_id,
            &members,
            "Membership",
            &coins(10, VOTE_DENOM),
        )
        .unwrap();

    let proxies: HashMap<_, _> = data
        .members
        .into_iter()
        .map(|member| {
            (
                member.owner_addr,
                ProxyContract::from_addr(Addr::unchecked(member.proxy_addr)),
            )
        })
        .collect();

    assert_eq!(proxies.len(), 2);

    assert!(
        membership
            .is_member(&app, proxies[members[0]].addr().as_str())
            .unwrap()
            .is_member
    );
    assert!(
        membership
            .is_member(&app, proxies[members[0]].addr().as_str())
            .unwrap()
            .is_member
    );
}
