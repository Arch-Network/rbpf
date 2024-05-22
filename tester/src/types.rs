use std::{cell::RefCell, collections::HashMap};

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct UtxoInfo {
    pub txid: String,
    pub vout: u32,
    pub authority: RefCell<Pubkey>,
    pub data: RefCell<Vec<u8>>,
}
impl UtxoInfo {
    pub fn id(&self) -> String {
        format!("{}:{}", self.txid, self.vout)
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Eq)]
pub struct Pubkey(pub Vec<u8>);
impl Pubkey {
    pub fn from_array(arr: [u8; 32]) -> Self {
        Pubkey(arr.to_vec())
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

#[derive(Clone, Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct Instruction {
    pub program_id: Pubkey,
    pub utxos: Vec<UtxoMeta>,
    pub data: Vec<u8>,
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
    pub instruction : Instruction,
    pub authority : HashMap<String, Vec<u8>>,
    pub data : HashMap<String, Vec<u8>>,
}