use thiserror::Error;

/// Reasons the program may fail
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ProgramError {
    /// Allows on-chain programs to implement program-specific error types and see them returned
    /// by the Solana runtime. A program-specific error may be any type that is represented as
    /// or serialized to a u32 integer.
    #[error("Custom program error: {0:#x}")]
    Custom(u32),
    #[error("The arguments provided to a program instruction were invalid")]
    InvalidArgument,
    #[error("An instruction's data contents was invalid")]
    InvalidInstructionData,
    #[error("An account's data contents was invalid")]
    InvalidAccountData,
    #[error("An account's data was too small")]
    AccountDataTooSmall,
    #[error("An account's balance was too small to complete the instruction")]
    InsufficientFunds,
    #[error("The account did not have the expected program id")]
    IncorrectProgramId,
    #[error("A signature was required but not found")]
    MissingRequiredSignature,
    #[error("An initialize instruction was sent to an account that has already been initialized")]
    AccountAlreadyInitialized,
    #[error("An attempt to operate on an account that hasn't been initialized")]
    UninitializedAccount,
    #[error("The instruction expected additional account keys")]
    NotEnoughAccountKeys,
    #[error("Failed to borrow a reference to account data, already borrowed")]
    AccountBorrowFailed,
    #[error("Length of the seed is too long for address generation")]
    MaxSeedLengthExceeded,
    #[error("Provided seeds do not result in a valid address")]
    InvalidSeeds,
    #[error("IO Error: {0}")]
    BorshIoError(String),
    #[error("An account does not have enough lamports to be rent-exempt")]
    AccountNotRentExempt,
    #[error("Unsupported sysvar")]
    UnsupportedSysvar,
    #[error("Provided owner is not allowed")]
    IllegalOwner,
    #[error("Accounts data allocations exceeded the maximum allowed per transaction")]
    MaxAccountsDataAllocationsExceeded,
    #[error("Account data reallocation was invalid")]
    InvalidRealloc,
    #[error("Instruction trace length exceeded the maximum allowed per transaction")]
    MaxInstructionTraceLengthExceeded,
    #[error("Builtin programs must consume compute units")]
    BuiltinProgramsMustConsumeComputeUnits,
    #[error("Invalid account owner")]
    InvalidAccountOwner,
    #[error("Program arithmetic overflowed")]
    ArithmeticOverflow,
    #[error("Account is immutable")]
    Immutable,
    #[error("Incorrect authority provided")]
    IncorrectAuthority,
}