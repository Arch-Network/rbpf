use core_types::{entrypoint, types::{Transaction, TxIn, TxOut,Pubkey,UtxoInfo}};
// use core_types::dentrypoint::{,};
entrypoint!(process_instruction);
use borsh::de::BorshDeserialize;
use core_types::{msg, types::{TransferInstruction}};
use bitcoin::{absolute::LockTime,transaction::{Transaction as BtcTransaction, Version}};
// use solana_program::entrypoint::ProgramResult;

// use rand;

// fn process_instruction(program_id: Pubkey, utxos : &[UtxoInfo], instruction_data : &Vec<u8>) -> Result<Transaction,String> {

//     let txin = TxIn {
//          txid: String::from("abcdef"),
//          vout:1,
//          script_sig: [12u8;32].to_vec(),
//          sequence: 5,
//          witness: vec![[12u8;32].to_vec(), [22u8;34].to_vec()]
//     };

//     let txout = TxOut {
//         amount: 10240,
//         script_pubkey: [122u8;64].to_vec(),
//     };

//     return Ok(Transaction {
//         version: 1,
//         input: vec![txin],
//         output: vec![txout],
//         lock_time: 15,
//     })
// }

// fn process_instruction(ins : Vec<u8>) -> Result<Transaction,String> {
//       let instruction = ProgramInstruction::try_from_slice(&ins).expect("error while reading in process_instrctuion");

//      let txin = TxIn {
//          txid: String::from("abcdef"),
//          vout:1,
//          script_sig: [12u8;32].to_vec(),
//          sequence: 5,
//          witness: vec![[12u8;32].to_vec(), [22u8;34].to_vec()]
//     };

//     let txout = TxOut {
//         amount: 10240,
//         script_pubkey: [122u8;64].to_vec(),
//     };

//     return Ok(Transaction {
//         version: 1,
//         input: vec![txin],
//         output: vec![txout],
//         lock_time: 15,
//     })
// }

pub fn process_instruction(
    key: Pubkey,
    accounts: &Vec<UtxoInfo>,
    ins: Vec<u8>,
) -> Result<Transaction, String> {
    let instruction = TransferInstruction::try_from_slice(&ins).expect("should be able to deserialise");
    // let a = vec![0;1025];
    let _ : Result<(), String> = match instruction {
        TransferInstruction::CpiTransfer(args) => Ok(()),
        TransferInstruction::ProgramTransfer(args) => {
            Ok(())
        }
    };

    msg!("Hello from msg");

    let txin = TxIn {
        txid: String::from("abcdef"),
        vout:1,
        script_sig: [12u8;32].to_vec(),
        sequence: 5,
        witness: vec![[12u8;32].to_vec(), [22u8;34].to_vec()]
    };

    let txout = TxOut {
        amount: 10240,
        script_pubkey: [122u8;64].to_vec(),
    };

    return Ok(Transaction {
        version: 1,
        input: vec![txin],
        output: vec![txout],
        lock_time: 15,
    })
}
