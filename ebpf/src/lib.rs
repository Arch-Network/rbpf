use tester::{entrypoint,types::*};

entrypoint!(process_instruction);

// use rand;

fn process_instruction(program_id: Pubkey, utxos : &[UtxoInfo], instruction_data : &Vec<u8>) -> Result<Transaction,String> {

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