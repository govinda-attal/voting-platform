use std::collections::HashMap;

use common::keys::{ATOM, VOTE_DENOM};
use common::msg::{ProposalMemberData, WithdrawableResp};
use cosmwasm_std::{coin, coins, Addr, Decimal};
use cw_multi_test::App;

use super::CodeId as MembershipId;
use distribution::multitest::{CodeId as DistributionId, Contract as DistributionContract};
use proposal::multitest::{CodeId as ProposalId, Contract as ProposalContract};
use proxy::multitest::{CodeId as ProxyId, Contract as ProxyContract};

#[test]
fn sample_member_vote_flow_from_exercise() {
    let admin = Addr::unchecked("admin");
    let member1 = Addr::unchecked("member1");
    let member2 = Addr::unchecked("member2");
    let member3 = Addr::unchecked("member3");
    let members = [member1.as_str(), member2.as_str(), member3.as_str()];
    let candidate = Addr::unchecked("candidate");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &admin, coins(100, VOTE_DENOM))
            .unwrap();

        router
            .bank
            .init_balance(storage, &candidate, coins(100, ATOM))
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
            Decimal::percent(19),
            coin(5, ATOM),
            coin(30, ATOM),
            proxy_id,
            proposal_id,
            distribution_id,
            &members,
            "Membership",
            &coins(100, VOTE_DENOM),
        )
        .unwrap();

    let membership_config = membership.load_config(&app);
    let distribution_contract =
        DistributionContract::from_addr(membership_config.distribution_contract);

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

    assert_eq!(proxies.len(), 3);
    let member1_proxy = proxies.get(members[0]).unwrap();
    let member2_proxy = proxies.get(members[1]).unwrap();
    let member3_proxy = proxies.get(members[2]).unwrap();

    assert!(
        membership
            .is_member(&app, member1_proxy.addr().as_str())
            .unwrap()
            .ok
    );
    assert!(
        membership
            .is_member(&app, member2_proxy.addr().as_str())
            .unwrap()
            .ok
    );

    assert_eq!(
        app.wrap().query_balance(&member1, VOTE_DENOM).unwrap(),
        coin(6, VOTE_DENOM),
    );

    assert_eq!(
        app.wrap().query_balance(&member2, VOTE_DENOM).unwrap(),
        coin(6, VOTE_DENOM),
    );
    assert_eq!(
        app.wrap().query_balance(&member3, VOTE_DENOM).unwrap(),
        coin(6, VOTE_DENOM),
    );

    let proposal_data = member1_proxy
        .propose_member(&mut app, &member1, &coins(5, VOTE_DENOM), &candidate)
        .unwrap()
        .unwrap();

    assert_eq!(proposal_data.owner_addr, candidate.to_string());

    let candidate_proposal =
        ProposalContract::from_addr(Addr::unchecked(proposal_data.proposal_addr));

    assert_eq!(
        app.wrap()
            .query_balance(candidate_proposal.addr(), VOTE_DENOM)
            .unwrap(),
        coin(5, VOTE_DENOM),
    );

    candidate_proposal
        .vote(&mut app, &member2, &coins(3, VOTE_DENOM))
        .unwrap();

    assert_eq!(
        app.wrap()
            .query_balance(candidate_proposal.addr(), VOTE_DENOM)
            .unwrap(),
        coin(8, VOTE_DENOM),
    );

    candidate_proposal
        .vote(&mut app, &member3, &coins(2, VOTE_DENOM))
        .unwrap();

    assert_eq!(
        app.wrap()
            .query_balance(candidate_proposal.addr(), VOTE_DENOM)
            .unwrap(),
        coin(10, VOTE_DENOM),
    );

    let candidate_proxy_data = candidate_proposal
        .join(&mut app, &candidate, &coins(100, ATOM))
        .unwrap()
        .unwrap();

    let candidate_now_member_proxy =
        ProxyContract::from_addr(Addr::unchecked(candidate_proxy_data.proxy_addr));

    assert!(
        membership
            .is_member(&app, candidate_now_member_proxy.addr().as_str())
            .unwrap()
            .ok
    );

    assert_eq!(
        app.wrap()
            .query_balance(candidate.as_str(), VOTE_DENOM)
            .unwrap(),
        coin(10, VOTE_DENOM),
    );

    assert_eq!(
        member1_proxy.withdrawable(&app).unwrap(),
        WithdrawableResp {
            funds: Some(coin(50, ATOM))
        }
    );

    assert_eq!(
        member2_proxy.withdrawable(&app).unwrap(),
        WithdrawableResp {
            funds: Some(coin(30, ATOM))
        }
    );

    assert_eq!(
        member3_proxy.withdrawable(&app).unwrap(),
        WithdrawableResp {
            funds: Some(coin(20, ATOM))
        }
    );

    member1_proxy.withdraw(&mut app, &member1).unwrap();

    assert_eq!(
        app.wrap().query_balance(&member1, ATOM).unwrap(),
        coin(50, ATOM),
    );

    assert_eq!(
        member1_proxy.withdrawable(&app).unwrap(),
        WithdrawableResp { funds: None }
    );

    assert_eq!(
        app.wrap()
            .query_balance(distribution_contract.addr(), ATOM)
            .unwrap(),
        coin(50, ATOM),
    );

    member2_proxy.buy_vote_tokens(&mut app, &member2).unwrap();

    assert_eq!(
        app.wrap().query_balance(&member2, VOTE_DENOM).unwrap(),
        coin(9, VOTE_DENOM),
    );
    assert_eq!(
        member2_proxy.withdrawable(&app).unwrap(),
        WithdrawableResp { funds: None }
    );

    assert_eq!(
        app.wrap()
            .query_balance(distribution_contract.addr(), ATOM)
            .unwrap(),
        coin(50, ATOM),
    );

    assert_eq!(
        distribution_contract.total_vote_tokens_in_circulation(&app),
        coin(25, VOTE_DENOM)
    );

    assert_eq!(
        app.wrap()
            .query_balance(distribution_contract.addr(), VOTE_DENOM)
            .unwrap(),
        coin(76, VOTE_DENOM),
    );
}

#[test]
fn member_vote_flow_with_rewards_and_vote_tokens_buy() {
    let admin = Addr::unchecked("admin");
    let alice = Addr::unchecked("alice");
    let bob = Addr::unchecked("bob");
    let members = [alice.as_str(), bob.as_str()];
    let charlie = Addr::unchecked("charlie");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &admin, coins(100, VOTE_DENOM))
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
            Decimal::percent(10),
            coin(5, ATOM),
            coin(30, ATOM),
            proxy_id,
            proposal_id,
            distribution_id,
            &members,
            "Membership",
            &coins(100, VOTE_DENOM),
        )
        .unwrap();

    let membership_config = membership.load_config(&app);
    let distribution_contract =
        DistributionContract::from_addr(membership_config.distribution_contract);

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
        coin(6, VOTE_DENOM),
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
            .query_balance(distribution_contract.addr(), ATOM)
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
            .query_balance(distribution_contract.addr(), ATOM)
            .unwrap(),
        coin(15, ATOM),
    );

    assert_eq!(
        distribution_contract.total_vote_tokens_in_circulation(&app),
        coin(13, VOTE_DENOM)
    );
}
