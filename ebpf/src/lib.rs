pub mod handler;

use anyhow::Result;
use core_types::{
    entrypoint,
    types::{Pubkey, Transaction, TxIn, TxOut, UtxoInfo},
};
// use core_types::dentrypoint::{,};
entrypoint!(process_instruction);
use bitcoin::{
    absolute::LockTime,
    transaction::{Transaction as BtcTransaction, Version},
};
use borsh::de::BorshDeserialize;
use core_types::{msg, types::TransferInstruction};

pub fn process_instruction(
    key: Pubkey,
    accounts: &Vec<UtxoInfo>,
    ins: Vec<u8>,
) -> Result<Transaction> {
    println!("Program ID: {:?}", key);
    println!("UtxoInfos: {:?}", accounts);
    println!("Instruction Data: {:?}", ins);
    let tx = handler::handler(&key, accounts, &ins)?;
    println!("Transaction: {:?}", tx);
    Ok(tx)

    // msg!("Hello from msg");

    // let txin = TxIn {
    //     txid: String::from("abcdef"),
    //     vout:1,
    //     script_sig: [15u8;32].to_vec(),
    //     sequence: 5,
    //     witness: vec![[15u8;32].to_vec(), [22u8;34].to_vec()]
    // };

    // let txout = TxOut {
    //     amount: 15250,
    //     script_pubkey: [155u8;64].to_vec(),
    // };

    // return Ok(Transaction {
    //     version: 1,
    //     input: vec![txin],
    //     output: vec![txout],
    //     lock_time: 15,
    // })
}
