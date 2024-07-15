use core_types::{
    entrypoint, program::set_return_data, types::{ProgramInstruction, Pubkey, Transaction, TxIn, TxOut}, utxo_info::UtxoInfo, UtxoIdentity
};
// use core_types::dentrypoint::{,};
use bitcoin::{
    absolute::LockTime,
    transaction::{Transaction as BtcTransaction, Version},
};
use borsh::de::BorshDeserialize;
use core_types::{msg, types::TransferInstruction,program::invoke};
// use solana_program::entrypoint::ProgramResult;

// use rand;

entrypoint!(process_instruction);
pub fn process_instruction(key: &Pubkey, utxos: &[UtxoInfo], ins: &[u8]) -> Result<(), String> {

    set_return_data("Hello".as_bytes());
    return Ok(());
}
