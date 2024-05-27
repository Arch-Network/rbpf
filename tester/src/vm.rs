// let utxo = UtxoMeta{ txid: String::from("a"), vout: 2 };
//     let instruction = Instruction {
//         program_id: Pubkey([1u8;32].to_vec()),
//         utxos: vec![utxo.clone()],
//         data: [12u8;1024].to_vec(),
//     };
//     let mut authority = HashMap::new();
//     authority.insert(String::from("a:2"), [12u8;78].to_vec());

//     let mut data: HashMap<String, Vec<u8>> = HashMap::new();
//     data.insert(String::from("a:2"), [12u8;78].to_vec());

//     let input_to_entrypoint = VmInput {
//         instruction, 
//         authority,
//         data
//     };
//     let mut serialised_data = borsh::to_vec(&input_to_entrypoint).unwrap();
//     let mut data_len = serialised_data.len() as u32;
//     let mut data_len = data_len.to_le_bytes().to_vec();
    
//     data_len.append(&mut serialised_data);

//     // to be sent inside the vm
//     let ptr = data_len.as_mut_ptr();

//     let size = unsafe { *(ptr as *mut u32)};
//     println!("The length of input is {}",size);
//     let data_slice = unsafe { from_raw_parts_mut(ptr.add(4), size as usize)};

//     let deserialised_output: () = from_slice(&data_slice).expect("abc");

//     let program_id: Pubkey = deserialised_output.instruction.program_id;
//             let authorities: HashMap<String, Vec<u8>> = deserialised_output.authority;
//             let data: HashMap<String, Vec<u8>> = deserialised_output.data;

//             let utxos = deserialised_output.instruction
//                 .utxos
//                 .iter()
//                 .map(|utxo| {
//                     use std::cell::RefCell;
//                     UtxoInfo {
//                         txid: utxo.txid.clone(),
//                         vout: utxo.vout,
//                         authority: RefCell::new(Pubkey(
//                             authorities
//                                 .get(&utxo.id())
//                                 .expect("this utxo does not exist in auth")
//                                 .to_vec(),
//                         )),
//                         data: RefCell::new(
//                             data.get(&utxo.id())
//                                 .expect("this utxo does not exist in data")
//                                 .to_vec(),
//                         ),
//                     }
//                 })
//                 .collect::<Vec<UtxoInfo>>();
//             let instruction_data: Vec<u8> = deserialised_output.instruction.data;

//             // println!("data inside the fucntion: {:?}\n",(program_id,instruction_data,utxos));

//             match process_instruction(program_id,&utxos,&instruction_data) {
//                 Ok(tx_hex) => {
//                     let mut new_authorities: HashMap<String, Vec<u8>> = HashMap::new();
//                     let mut new_data: HashMap<String, Vec<u8>> = HashMap::new();
//                     utxos.iter().for_each(|utxo| {
//                         new_authorities.insert(utxo.id(), utxo.authority.clone().into_inner().0);
//                         new_data.insert(utxo.id(), utxo.data.clone().into_inner());
//                     });
//                    let mut serialised_output = borsh::to_vec(&(new_authorities, new_data, tx_hex)).unwrap();
//                     let output_len = serialised_output.len();
//                     println!("output length is {}", output_len);
//                     unsafe {*(ptr as *mut u32) = output_len as u32;}

//                     unsafe {
//                         std::ptr::copy_nonoverlapping(serialised_output.as_mut_ptr(),ptr.add(4),output_len);
//                     }
//                 }
//                 Err(e) => {
//                     println!("Failed at process instrcution")
//                 }
//             }
// //  Checking if the correct data can be read outside
// let size = unsafe { *(ptr as *mut u32)};
//     println!("The length of input is {}",size);
//     let data_slice = unsafe { from_raw_parts_mut(ptr.add(4), size as usize)};

//     let deserialised_output: (HashMap<String,Vec<u8>>,HashMap<String,Vec<u8>>,Transaction) = from_slice(&data_slice).expect("abc");
    
//     println!("\n\nFInal {:?}\n",deserialised_output);

// }

