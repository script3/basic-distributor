use soroban_sdk::{contracttype, unwrap::UnwrapOptimized, Address, Env, Symbol};

//********** Storage Keys **********//

const IS_INIT_KEY: &str = "IsInit";
const ADMIN_KEY: &str = "Admin";
const TOKEN_KEY: &str = "Token";
const DEADLINE_KEY: &str = "Deadline";
const FINALIZED_KEY: &str = "Final";

#[derive(Clone)]
#[contracttype]
pub enum DistributorKey {
    Claim(Address),
    Dist(Address),
}

//********** Storage Utils **********//

pub const ONE_DAY_LEDGERS: u32 = 17280; // assumes 5 seconds per ledger on average

const LEDGER_BUMP_SHARED: u32 = 31 * ONE_DAY_LEDGERS;
const LEDGER_THRESHOLD_SHARED: u32 = LEDGER_BUMP_SHARED - ONE_DAY_LEDGERS;

const LEDGER_BUMP_MAX_DEADLINE: u32 = 91 * ONE_DAY_LEDGERS;

/// Bump the instance lifetime by the defined amount
pub fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED);
}

/********** Instance **********/

/// Check if the contract has been initialized
pub fn get_is_init(e: &Env) -> bool {
    e.storage().instance().has(&Symbol::new(e, IS_INIT_KEY))
}

/// Set the contract as initialized
pub fn set_is_init(e: &Env) {
    e.storage()
        .instance()
        .set::<Symbol, bool>(&Symbol::new(e, IS_INIT_KEY), &true);
}

/// Check if the contract has been initialized
pub fn is_finalized(e: &Env) -> bool {
    e.storage().instance().has(&Symbol::new(e, FINALIZED_KEY))
}

/// Set the contract as initialized
pub fn set_finalized(e: &Env) {
    e.storage()
        .instance()
        .set::<Symbol, bool>(&Symbol::new(e, FINALIZED_KEY), &true);
}

/// Get the owner of the distribution
pub fn get_admin(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&Symbol::new(e, ADMIN_KEY))
        .unwrap_optimized()
}

/// Set the owner of the distribution
pub fn set_admin(e: &Env, admin: &Address) {
    e.storage()
        .instance()
        .set::<Symbol, Address>(&Symbol::new(e, ADMIN_KEY), &admin);
}

/// Get the token for distribution
pub fn get_token(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&Symbol::new(e, TOKEN_KEY))
        .unwrap_optimized()
}

/// Set the token for distribution
pub fn set_token(e: &Env, token: &Address) {
    e.storage()
        .instance()
        .set::<Symbol, Address>(&Symbol::new(e, TOKEN_KEY), &token);
}

/// Get the deadline for distribution
pub fn get_deadline(e: &Env) -> u32 {
    e.storage()
        .instance()
        .get(&Symbol::new(e, DEADLINE_KEY))
        .unwrap_optimized()
}

/// Set the deadline for distribution
pub fn set_deadline(e: &Env, ledger: &u32) {
    e.storage()
        .instance()
        .set::<Symbol, u32>(&Symbol::new(e, DEADLINE_KEY), &ledger);
}

/********** Temporary **********/

/// Check if someone has claimed
pub fn has_claimed(e: &Env, user: &Address) -> bool {
    let key = DistributorKey::Claim(user.clone());
    e.storage().temporary().has(&key)
}

/// Set that a user has claimed the distribution
pub fn set_claimed(e: &Env, user: &Address) {
    let key = DistributorKey::Claim(user.clone());
    e.storage()
        .temporary()
        .set::<DistributorKey, bool>(&key, &true);
    e.storage()
        .temporary()
        .extend_ttl(&key, LEDGER_BUMP_MAX_DEADLINE, LEDGER_BUMP_MAX_DEADLINE);
}

/// Get the distribution for a user
pub fn get_distribution(e: &Env, user: &Address) -> i128 {
    let key = DistributorKey::Dist(user.clone());
    e.storage().temporary().get(&key).unwrap_or(0)
}

/// Set the distribution for a user
pub fn set_distribution(e: &Env, user: &Address, amount: i128) {
    let key = DistributorKey::Dist(user.clone());
    e.storage().temporary().set(&key, &amount);
    e.storage()
        .temporary()
        .extend_ttl(&key, LEDGER_BUMP_MAX_DEADLINE, LEDGER_BUMP_MAX_DEADLINE);
}
