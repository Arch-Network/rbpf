use borsh::{BorshDeserialize, BorshSerialize};
use std::convert::AsRef;
use std::{cell::RefCell, collections::HashMap, hash::Hash};

use crate::stable_vec::StableVec;
use crate::UtxoId;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Eq, Default, Hash, Copy)]
pub struct Pubkey(pub [u8; 32]);
impl Pubkey {
    pub fn from_array(arr: [u8; 32]) -> Self {
        Pubkey(arr)
    }

    pub const fn to_bytes(self) -> [u8; 32] {
        self.0
    }
}

impl AsRef<[u8]> for Pubkey {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl AsMut<[u8]> for Pubkey {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0[..]
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct Transaction {
    pub version: u32,
    pub input: Vec<TxIn>,
    pub output: Vec<TxOut>,
    pub lock_time: u32,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct TxIn {
    pub txid: String,
    pub vout: u32,
    pub script_sig: Vec<u8>,
    pub sequence: u32,
    pub witness: Vec<Vec<u8>>,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct TxOut {
    pub amount: u64,
    pub script_pubkey: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize, Default)]
pub struct Instruction {
    pub program_id: Pubkey,
    pub utxos: Vec<u16>,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ProgramInstruction {
    pub program_id: Pubkey,
    pub utxos: Vec<UtxoId>,
    pub data: Vec<u8>,
}

#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct StableInstruction {
    pub utxos: StableVec<UtxoId>,
    pub data: StableVec<u8>,
    pub program_id: Pubkey,
}

impl From<ProgramInstruction> for StableInstruction {
    fn from(other: ProgramInstruction) -> Self {
        Self {
            utxos: other.utxos.into(),
            data: other.data.into(),
            program_id: other.program_id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct UtxoMeta {
    pub txid: String,
    pub vout: u32,
}

impl UtxoMeta {
    pub fn id(&self) -> String {
        format!("{}:{}", self.txid, self.vout)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct UnsignedTransaction {
    pub version: u32,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub locktime: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct Input {
    pub txid: String,
    pub vout: u32,
    pub sequence: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct Output {
    pub value: u64,
    pub address: String,
}

#[derive(Clone, Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct VmInput {
    pub instruction: Instruction,
    pub authority: HashMap<String, Vec<u8>>,
    pub data: HashMap<String, Vec<u8>>,
}

// TODO: Delete
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum TransferInstruction {
    CpiTransfer(u64),
    ProgramTransfer(u64),
}

#[macro_export]
macro_rules! msg {
    ($msg:expr) => {
        $crate::types::sol_log($msg)
    };
    ($($arg:tt)*) => ($crate::types::sol_log(&format!($($arg)*)));
}

/// Print a string to the log.
#[inline]
pub fn sol_log(message: &str) {
    unsafe {
        crate::types::sol_log_(message.as_ptr(), message.len() as u64);
    }
}

macro_rules! define_syscall {
	(fn $name:ident($($arg:ident: $typ:ty),*) -> $ret:ty) => {
		extern "C" {
			pub fn $name($($arg: $typ),*) -> $ret;
		}
	};
	(fn $name:ident($($arg:ident: $typ:ty),*)) => {
		define_syscall!(fn $name($($arg: $typ),*) -> ());
	}
}

define_syscall!(fn sol_log_(message: *const u8, len: u64));
