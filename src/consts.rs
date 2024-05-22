/// Maximum number of bytes a program may add to an account during a single realloc
pub const MAX_PERMITTED_DATA_INCREASE: usize = 1_024 * 10;

/// Maximum number of instruction accounts that can be serialized into the
/// SBF VM.
pub const MAX_INSTRUCTION_ACCOUNTS: u8 = 255;

/// `assert_eq(std::mem::align_of::<u128>(), 8)` is true for BPF but not for some host machines
pub const BPF_ALIGN_OF_U128: usize = 8;

/// Value used to indicate that a serialized account is not a duplicate
pub const NON_DUP_MARKER: u8 = u8::MAX;

pub const PUBKEY_BYTES: u8 = 32;
