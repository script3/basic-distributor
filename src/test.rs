#![cfg(test)]

use crate::{
    errors::ContractError, storage::ONE_DAY_LEDGERS, testutils::EnvTestUtils, DistributorClient,
};
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events},
    token::{StellarAssetClient, TokenClient},
    vec, Address, Env, Error, IntoVal, Symbol,
};

mod distributor_wasm {
    soroban_sdk::contractimport!(
        file = "./target/wasm32-unknown-unknown/optimized/basic_distributor.wasm"
    );
}

#[test]
fn test_distribute() {
    let env = Env::default();
    env.set_default_info();
    env.mock_all_auths();

    let dist_id = env.register_contract_wasm(None, distributor_wasm::WASM);
    let dist_client = DistributorClient::new(&env, &dist_id);

    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let token_setup_client = StellarAssetClient::new(&env, &token);
    let token_client = TokenClient::new(&env, &token);
    let admin = Address::generate(&env);
    let deadline = env.ledger().sequence() + 90 * ONE_DAY_LEDGERS;

    let addr1 = Address::generate(&env);
    let amount1: i128 = 1342345;
    let addr2 = Address::generate(&env);
    let amount2: i128 = 89657234523425;
    let addr3 = Address::generate(&env);
    let amount3: i128 = 7823412341;
    let addr4_no_claim = Address::generate(&env);
    let amount4: i128 = 234521124;
    let addr5 = Address::generate(&env);
    let amount5: i128 = 43512351345;
    let addr6_no_dist = Address::generate(&env);

    let total_amount = amount1 + amount2 + amount3 + amount4 + amount5;
    token_setup_client.mint(&dist_id, &total_amount);

    dist_client.initialize(&token, &deadline, &admin);

    dist_client.set_distribution(&vec![
        &env,
        (addr1.clone(), amount1),
        (addr2.clone(), amount2),
        (addr3.clone(), amount3),
        (addr4_no_claim.clone(), amount4),
    ]);
    dist_client.set_distribution(&vec![&env, (addr5.clone(), amount5)]);

    env.jump(ONE_DAY_LEDGERS);

    // verify claim is blocked pre-finalize
    let result = dist_client.try_claim(&addr1);
    assert_eq!(
        result.err(),
        Some(Ok(Error::from_contract_error(
            ContractError::NotFinalizedError as u32
        )))
    );

    dist_client.finalize();

    // verify finalize and set_distribution cannot be called again
    let result = dist_client.try_finalize();
    assert_eq!(
        result.err(),
        Some(Ok(Error::from_contract_error(
            ContractError::AlreadyFinalizedError as u32
        )))
    );

    let result = dist_client.try_set_distribution(&vec![&env, (addr6_no_dist.clone(), 1)]);
    assert_eq!(
        result.err(),
        Some(Ok(Error::from_contract_error(
            ContractError::AlreadyFinalizedError as u32
        )))
    );

    // verify claim
    let claim_amount_1 = dist_client.claim(&addr1);
    assert_eq!(claim_amount_1, amount1);

    // claim - validate auth
    assert_eq!(
        env.auths()[0],
        (
            addr1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    dist_id.clone(),
                    Symbol::new(&env, "claim"),
                    vec![&env, addr1.to_val()]
                )),
                sub_invocations: std::vec![]
            }
        )
    );

    // claim - validate events
    let events = env.events().all();
    let tx_events = vec![&env, events.last().unwrap()];
    assert_eq!(
        tx_events,
        vec![
            &env,
            (
                dist_id.clone(),
                (Symbol::new(&env, "dist_claim"), addr1.clone()).into_val(&env),
                claim_amount_1.into_val(&env)
            )
        ]
    );

    // claim - validate chain results
    assert_eq!(token_client.balance(&addr1), amount1);
    assert_eq!(token_client.balance(&dist_id), total_amount - amount1);
    assert_eq!(dist_client.get_claimed(&addr1), true);
    assert_eq!(dist_client.get_claimed(&addr2), false);

    env.jump(89 * ONE_DAY_LEDGERS);

    assert_eq!(env.ledger().sequence(), deadline);

    // verify claim cannot be re-run
    let result = dist_client.try_claim(&addr1);
    assert_eq!(
        result.err(),
        Some(Ok(Error::from_contract_error(
            ContractError::AlreadyClaimedError as u32
        )))
    );

    // verify claim errors for user without distribution
    let result = dist_client.try_claim(&addr6_no_dist);
    assert_eq!(
        result.err(),
        Some(Ok(Error::from_contract_error(
            ContractError::NoDistributionError as u32
        )))
    );

    // claim for all users but addr4_no_claim
    let claim_amount_2 = dist_client.claim(&addr2);
    let claim_amount_3 = dist_client.claim(&addr3);
    let claim_amount_5 = dist_client.claim(&addr5);

    // verify unclaimed tokens cannot be removed until after the deadline
    let result = dist_client.try_refund();
    assert_eq!(
        result.err(),
        Some(Ok(Error::from_contract_error(
            ContractError::DeadlineError as u32
        )))
    );

    env.jump(1);

    // verify claim fails after deadline
    let result = dist_client.try_claim(&addr4_no_claim);
    assert_eq!(
        result.err(),
        Some(Ok(Error::from_contract_error(
            ContractError::DeadlineError as u32
        )))
    );

    // verify refund
    let refund_amount = dist_client.refund();

    // refund - verify auth
    assert_eq!(env.auths().len(), 0);

    // ***** verify tokens are correctly distributed *****

    assert_eq!(token_client.balance(&addr1), amount1);
    assert_eq!(amount1, claim_amount_1);
    assert_eq!(token_client.balance(&addr2), amount2);
    assert_eq!(amount2, claim_amount_2);
    assert_eq!(token_client.balance(&addr3), amount3);
    assert_eq!(amount3, claim_amount_3);
    assert_eq!(token_client.balance(&addr4_no_claim), 0);
    assert_eq!(token_client.balance(&addr5), amount5);
    assert_eq!(amount5, claim_amount_5);
    assert_eq!(token_client.balance(&dist_id), 0);
    assert_eq!(token_client.balance(&admin), refund_amount);
    assert_eq!(refund_amount, amount4);
}

#[test]
fn test_valid_initialize() {
    let env = Env::default();
    env.set_default_info();
    env.mock_all_auths();

    let dist_id = env.register_contract_wasm(None, distributor_wasm::WASM);
    let dist_client = DistributorClient::new(&env, &dist_id);

    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let admin = Address::generate(&env);

    let low_deadline = env.ledger().sequence() + 30 * ONE_DAY_LEDGERS - 1;
    let result_low = dist_client.try_initialize(&token, &low_deadline, &admin);
    assert_eq!(
        result_low.err(),
        Some(Ok(Error::from_contract_error(
            ContractError::DeadlineError as u32
        )))
    );

    let high_deadline = env.ledger().sequence() + 90 * ONE_DAY_LEDGERS + 1;
    let result_high = dist_client.try_initialize(&token, &high_deadline, &admin);
    assert_eq!(
        result_high.err(),
        Some(Ok(Error::from_contract_error(
            ContractError::DeadlineError as u32
        )))
    );

    dist_client.initialize(&token, &(low_deadline + 1), &admin);

    assert_eq!(dist_client.get_admin(), admin);
    assert_eq!(dist_client.get_deadline(), low_deadline + 1);
    assert_eq!(dist_client.get_token(), token);
}

#[test]
fn test_admin_only_functions() {
    let env = Env::default();
    env.set_default_info();
    env.mock_all_auths();

    let dist_id = env.register_contract_wasm(None, distributor_wasm::WASM);
    let dist_client = DistributorClient::new(&env, &dist_id);

    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let token_setup_client = StellarAssetClient::new(&env, &token);
    let token_client = TokenClient::new(&env, &token);
    let admin = Address::generate(&env);
    let deadline = env.ledger().sequence() + 45 * ONE_DAY_LEDGERS;

    dist_client.initialize(&token, &deadline, &admin);

    let amount = 123145;
    token_setup_client.mint(&dist_id, &amount);
    let addr1 = Address::generate(&env);
    let distributions = vec![&env, (addr1.clone(), amount)];

    dist_client.set_distribution(&distributions);

    // validate auth
    assert_eq!(
        env.auths()[0],
        (
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    dist_id.clone(),
                    Symbol::new(&env, "set_distribution"),
                    vec![&env, distributions.to_val()]
                )),
                sub_invocations: std::vec![]
            }
        )
    );

    dist_client.finalize();

    // validate auth
    assert_eq!(
        env.auths()[0],
        (
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    dist_id.clone(),
                    Symbol::new(&env, "finalize"),
                    vec![&env]
                )),
                sub_invocations: std::vec![]
            }
        )
    );

    env.jump(45 * ONE_DAY_LEDGERS + 1);

    let new_admin = Address::generate(&env);
    dist_client.set_admin(&new_admin);

    // validate auth
    assert_eq!(
        env.auths()[0],
        (
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    dist_id.clone(),
                    Symbol::new(&env, "set_admin"),
                    vec![&env, new_admin.into_val(&env)]
                )),
                sub_invocations: std::vec![]
            }
        )
    );

    // validate chain
    assert_eq!(dist_client.get_admin(), new_admin);

    // validate refund goes to current admin
    let refund_amount = dist_client.refund();
    assert_eq!(token_client.balance(&new_admin), refund_amount);
    assert_eq!(token_client.balance(&admin), 0);
    assert_eq!(refund_amount, amount);
}
