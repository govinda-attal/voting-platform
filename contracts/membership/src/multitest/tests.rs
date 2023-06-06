use std::collections::HashMap;

use common::keys::{ATOM, VOTE_DENOM};
use common::msg::{ProposalMemberData, WithdrawableResp};
use cosmwasm_std::{coin, coins, Addr};
use cw_multi_test::App;

use super::CodeId as MembershipId;
use distribution::multitest::CodeId as DistributionId;
use proposal::multitest::{CodeId as ProposalId, Contract as ProposalContract};
use proxy::multitest::{CodeId as ProxyId, Contract as ProxyContract};

#[test]
fn test_member_vote_flow() {
    let admin = Addr::unchecked("admin");
    let alice = Addr::unchecked("alice");
    let bob = Addr::unchecked("bob");
    let members = [alice.as_str(), bob.as_str()];
    let charlie = Addr::unchecked("charlie");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &admin, coins(10, VOTE_DENOM))
            .unwrap();

        router
            .bank
            .init_balance(storage, &charlie, coins(30, ATOM))
            .unwrap();
    });

    let proxy_id = ProxyId::store_code(&mut app);
    let proposal_id = ProposalId::store_code(&mut app);
    let distribution_id = DistributionId::store_code(&mut app);
    let membership_id = MembershipId::store_code(&mut app);

    let (membership, data) = membership_id
        .instantiate(
            &mut app,
            &admin,
            coin(2, VOTE_DENOM),
            coin(5, ATOM),
            coin(30, ATOM),
            proxy_id,
            proposal_id,
            distribution_id,
            &members,
            "Membership",
            &coins(10, VOTE_DENOM),
        )
        .unwrap();

    let membership_config = membership.load_config(&app);

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
    let alice_proxy = proxies.get(members[0]).unwrap();
    let bob_proxy = proxies.get(members[1]).unwrap();

    assert!(
        membership
            .is_member(&app, alice_proxy.addr().as_str())
            .unwrap()
            .ok
    );
    assert!(
        membership
            .is_member(&app, bob_proxy.addr().as_str())
            .unwrap()
            .ok
    );

    assert_eq!(
        app.wrap().query_balance(&alice, VOTE_DENOM).unwrap(),
        coin(5, VOTE_DENOM),
    );

    assert_eq!(
        app.wrap().query_balance(&bob, VOTE_DENOM).unwrap(),
        coin(5, VOTE_DENOM),
    );

    let proposal_data = alice_proxy
        .propose_member(&mut app, &alice, &coins(3, VOTE_DENOM), &charlie)
        .unwrap()
        .unwrap();

    assert_eq!(proposal_data.owner_addr, charlie.to_string());

    let charlie_proposal =
        ProposalContract::from_addr(Addr::unchecked(proposal_data.proposal_addr));

    assert_eq!(
        app.wrap()
            .query_balance(charlie_proposal.addr(), VOTE_DENOM)
            .unwrap(),
        coin(3, VOTE_DENOM),
    );

    charlie_proposal
        .vote(&mut app, &bob, &coins(3, VOTE_DENOM))
        .unwrap();

    assert_eq!(
        app.wrap()
            .query_balance(charlie_proposal.addr(), VOTE_DENOM)
            .unwrap(),
        coin(6, VOTE_DENOM),
    );

    let charlie_proxy_data = charlie_proposal
        .join(&mut app, &charlie, &coins(30, ATOM))
        .unwrap()
        .unwrap();

    let charlie_proxy = ProxyContract::from_addr(Addr::unchecked(charlie_proxy_data.proxy_addr));

    assert!(
        membership
            .is_member(&app, charlie_proxy.addr().as_str())
            .unwrap()
            .ok
    );

    assert_eq!(
        app.wrap().query_balance(&charlie, VOTE_DENOM).unwrap(),
        coin(2, VOTE_DENOM),
    );

    assert_eq!(
        alice_proxy.withdrawable(&app).unwrap(),
        WithdrawableResp {
            funds: Some(coin(15, ATOM))
        }
    );

    assert_eq!(
        bob_proxy.withdrawable(&app).unwrap(),
        WithdrawableResp {
            funds: Some(coin(15, ATOM))
        }
    );

    alice_proxy.withdraw(&mut app, &alice).unwrap();

    assert_eq!(
        app.wrap().query_balance(&alice, ATOM).unwrap(),
        coin(15, ATOM),
    );

    assert_eq!(
        alice_proxy.withdrawable(&app).unwrap(),
        WithdrawableResp { funds: None }
    );

    assert_eq!(
        app.wrap()
            .query_balance(&membership_config.distribution_contract, ATOM)
            .unwrap(),
        coin(15, ATOM),
    );

    bob_proxy.buy_vote_tokens(&mut app, &bob).unwrap();

    assert_eq!(
        app.wrap().query_balance(&bob, VOTE_DENOM).unwrap(),
        coin(5, VOTE_DENOM),
    );
    assert_eq!(
        bob_proxy.withdrawable(&app).unwrap(),
        WithdrawableResp { funds: None }
    );

    assert_eq!(
        app.wrap()
            .query_balance(&membership_config.distribution_contract, ATOM)
            .unwrap(),
        coin(15, ATOM),
    );

    assert_eq!(
        app.wrap()
            .query_balance(&membership_config.distribution_contract, VOTE_DENOM)
            .unwrap(),
        coin(1, VOTE_DENOM),
    );
}
