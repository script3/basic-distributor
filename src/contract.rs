use soroban_sdk::{
    assert_with_error, contract, contractimpl, token::TokenClient, Address, Env, Vec,
};

use crate::{
    errors::ContractError,
    events::ContractEvents,
    storage::{self, ONE_DAY_LEDGERS},
};

#[contract]
pub struct Distributor;

#[contractimpl]
impl Distributor {
    /// Initialize the contract
    ///
    /// ### Arguments
    /// * `token` - The token to distribute
    /// * `deadline` - The deadline ledger sequence number of the distribution
    /// * `admin` - The admin of the contract
    ///
    /// ### Panics
    /// * `AlreadyInitializedError` - If the contract has already been initialized
    /// * `DeadlineError` - If the deadline is not withing [30, 90] days of ledgers in the future
    ///                     assuming 5s a ledger
    pub fn initialize(e: Env, token: Address, deadline: u32, admin: Address) {
        assert_with_error!(
            &e,
            !storage::get_is_init(&e),
            ContractError::AlreadyInitializedError
        );
        assert_with_error!(
            &e,
            deadline >= e.ledger().sequence() + 30 * ONE_DAY_LEDGERS
                && deadline <= e.ledger().sequence() + 90 * ONE_DAY_LEDGERS,
            ContractError::DeadlineError
        );

        storage::set_token(&e, &token);
        storage::set_deadline(&e, &deadline);
        storage::set_admin(&e, &admin);

        storage::set_is_init(&e);
    }

    //********** Read-Only ***********//

    /// Fetch it a user has claimed their distribution
    pub fn get_claimed(e: Env, user: Address) -> bool {
        storage::has_claimed(&e, &user)
    }

    /// Fetch the deadline ledger sequence for the distribution
    pub fn get_deadline(e: Env) -> u32 {
        storage::get_deadline(&e)
    }

    /// Fetch the admin of the contract
    pub fn get_admin(e: Env) -> Address {
        storage::get_admin(&e)
    }

    /// Fetch the token being distributed
    pub fn get_token(e: Env) -> Address {
        storage::get_token(&e)
    }

    //********** Read-Write ***********//

    /// (Admin Only) Set the distribution for users
    ///
    /// ### Arguments
    /// * `distributions` - The distributions to set
    ///
    /// ### Panics
    /// * `AlreadyFinalizedError` - If the contract has already been finalized
    pub fn set_distribution(e: Env, distributions: Vec<(Address, i128)>) {
        storage::get_admin(&e).require_auth();
        assert_with_error!(
            &e,
            !storage::is_finalized(&e),
            ContractError::AlreadyFinalizedError
        );
        storage::extend_instance(&e);

        for (user, amount) in distributions {
            storage::set_distribution(&e, &user, amount);
        }
    }

    /// (Admin Only) Finalize the distribution
    ///
    /// ### Panics
    /// * `AlreadyFinalizedError` - If the contract has already been finalized
    pub fn finalize(e: Env) {
        storage::get_admin(&e).require_auth();

        assert_with_error!(
            &e,
            !storage::is_finalized(&e),
            ContractError::AlreadyFinalizedError
        );
        storage::extend_instance(&e);

        storage::set_finalized(&e);
    }

    /// (Admin Only) Set the admin of the contract
    ///
    /// ### Arguments
    /// * `admin` - The new admin of the contract
    pub fn set_admin(e: Env, admin: Address) {
        storage::get_admin(&e).require_auth();
        storage::set_admin(&e, &admin);
    }

    /// Claim the distribution
    ///
    /// ### Arguments
    /// * `user` - The user to claim the distribution for
    ///
    /// ### Panics
    /// * `NotFinalizedError` - If the contract has not been finalized
    /// * `AlreadyClaimedError` - If the user has already claimed their distribution
    /// * `DeadlineError` - If the deadline has passed
    /// * `NoDistributionError` - If the user has no distribution to claim
    pub fn claim(e: Env, user: Address) -> i128 {
        user.require_auth();
        assert_with_error!(
            &e,
            storage::is_finalized(&e),
            ContractError::NotFinalizedError
        );
        assert_with_error!(
            &e,
            !storage::has_claimed(&e, &user),
            ContractError::AlreadyClaimedError
        );
        assert_with_error!(
            &e,
            e.ledger().sequence() <= storage::get_deadline(&e),
            ContractError::DeadlineError
        );
        storage::extend_instance(&e);

        let amount = storage::get_distribution(&e, &user);
        assert_with_error!(&e, amount > 0, ContractError::NoDistributionError);

        storage::set_claimed(&e, &user);

        let token = storage::get_token(&e);
        TokenClient::new(&e, &token).transfer(&e.current_contract_address(), &user, &amount);

        ContractEvents::claim(&e, user, amount);
        amount
    }

    /// Refund the remaining balance to the admin
    ///
    /// ### Panics
    /// * `DeadlineError` - If the deadline has not passed
    pub fn refund(e: Env) -> i128 {
        assert_with_error!(
            &e,
            e.ledger().sequence() > storage::get_deadline(&e),
            ContractError::DeadlineError
        );
        let token = storage::get_token(&e);
        let token_client = TokenClient::new(&e, &token);

        let balance = token_client.balance(&e.current_contract_address());

        if balance > 0 {
            let admin = storage::get_admin(&e);
            token_client.transfer(&e.current_contract_address(), &admin, &balance);
        }

        balance
    }
}
