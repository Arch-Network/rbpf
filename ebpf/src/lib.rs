use core_types::{
    entrypoint,
    types::{Pubkey, Transaction, TxIn, TxOut},
    utxo_info::UtxoInfo,
};
// use core_types::dentrypoint::{,};
use bitcoin::{
    absolute::LockTime,
    transaction::{Transaction as BtcTransaction, Version},
};
use borsh::de::BorshDeserialize;
use core_types::{msg, types::TransferInstruction};
// use solana_program::entrypoint::ProgramResult;

// use rand;

entrypoint!(process_instruction);
pub fn process_instruction(key: &Pubkey, utxos: &[UtxoInfo], ins: &[u8]) -> Result<(), String> {
    let first_utxo = &utxos[0];
    // first_utxo.realloc(15, false);
    // first_utxo.try_borrow_mut_data().unwrap().as_mut().copy_from_slice(&[15u8;15]);
    let data = first_utxo.try_borrow_mut_data();

    return Ok(());
}
