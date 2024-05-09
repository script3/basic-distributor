use soroban_sdk::{Address, Env, Symbol};
pub struct ContractEvents {}

impl ContractEvents {
    /// Emitted when a distribution is claimed
    ///
    /// - topics - `["dist_claim", user: Address]`
    /// - data - `amount: i128`
    pub fn claim(e: &Env, user: Address, amount: i128) {
        let topics = (Symbol::new(&e, "dist_claim"), user);
        e.events().publish(topics, amount);
    }
}
