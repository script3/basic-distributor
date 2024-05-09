use soroban_sdk::contracterror;

/// The error codes for the contract.
#[contracterror]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ContractError {
    // Default errors to align with built-in contract
    InternalError = 1,
    AlreadyInitializedError = 3,

    UnauthorizedError = 4,

    NegativeAmountError = 8,
    AllowanceError = 9,
    BalanceError = 10,
    OverflowError = 12,

    // Custom errors
    DeadlineError = 100,
    NotFinalizedError = 101,
    AlreadyClaimedError = 102,
    AlreadyFinalizedError = 103,
    NoDistributionError = 104,
}
