use core_types::{entrypoint, types::{Transaction, TxIn, TxOut,Pubkey}, utxo_info::UtxoInfo};
// use core_types::dentrypoint::{,};
use borsh::de::BorshDeserialize;
use core_types::{msg, types::{TransferInstruction}};
use bitcoin::{absolute::LockTime,transaction::{Transaction as BtcTransaction, Version}};
// use solana_program::entrypoint::ProgramResult;

// use rand;

entrypoint!(process_instruction);
pub fn process_instruction(
    key: &Pubkey,
    utxos: &[UtxoInfo],
    ins: &[u8],
) -> Result<(), String> {


    msg!("Hello from msg");

    return Ok(())
}
