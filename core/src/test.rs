// use crate::entrypoint;
//     #[test]
//     fn test_entrypoint() {
//         println!("Hello");
//         // #[global_allocator]
//         // static A: BumpAllocator = BumpAllocator {
//         //     start: HEAP_START_ADDRESS as usize,
//         //     len: HEAP_LENGTH,
//         // };

//         entrypoint!(process_instruction);

//         }

//     pub fn process_instruction(program_id: Pubkey, utxos : &[UtxoInfo], instruction_data : &Vec<u8>) -> Result<(),String> {

//         // let txin = TxIn {
//         //      txid: String::from("abcdef"),
//         //      vout:1,
//         //      script_sig: [12u8;32].to_vec(),
//         //      sequence: 5,
//         //      witness: vec![[12u8;32].to_vec(), [22u8;34].to_vec()]
//         // };
    
//         // let txout = TxOut {
//         //     amount: 10240,
//         //     script_pubkey: [122u8;64].to_vec(),
//         // };
    
//         // return Ok(Transaction {
//         //     version: 1,
//         //     input: vec![txin],
//         //     output: vec![txout],
//         //     lock_time: 15,
//         // })
//         Ok(())
//     }

//     pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
//         use std::collections::HashMap;
//         let (program_id, utxos, instruction_data) =
//             unsafe { deserialize(input) };
//         match process_instruction(program_id, &utxos, &instruction_data) {
//             Ok(tx_hex) => {
//                 let mut new_authorities: HashMap<String, Vec<u8>> = HashMap::new();
//                 let mut new_data: HashMap<String, Vec<u8>> = HashMap::new();
//                 utxos.iter().for_each(|utxo| {
//                     new_authorities.insert(utxo.id(), utxo.authority.clone().into_inner().0);
//                     new_data.insert(utxo.id(), utxo.data.clone().into_inner());
//                 });
//                let mut serialised_output = borsh::to_vec(&(new_authorities, new_data, tx_hex)).unwrap();
//                 let output_len = serialised_output.len();
//                 println!("output length is {}", output_len);
//                 unsafe {*(input as *mut u32) = output_len as u32;}

//                 unsafe {
//                     std::ptr::copy_nonoverlapping(serialised_output.as_mut_ptr(),input.add(4),output_len);
//                 }
//                 return 0;
//             }
//             Err(e) => {
//                 return 1;
//             }
//         }
// }
