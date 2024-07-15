use std::{collections::HashMap, fs::File, io::Read, str::from_utf8, string, sync::Arc};
use borsh::try_from_slice_with_schema;
use core_types::{types::*, UtxoIdentity};
use sha256::digest;
use solana_program::address_lookup_table::instruction;
use solana_rbpf::{
    aligned_memory::AlignedMemory, ebpf, elf::Executable, memory_region::{MemoryMapping, MemoryRegion}, program::{BuiltinProgram, FunctionRegistry}, verifier::RequisiteVerifier, vm::{Config, EbpfVm, TestContextObject}
};
use core_types::types::*;
use crate::{config::create_program_runtime_environment_v1, processor::{Message, MessageProcessor, TransactionContext, UtxoSharedData}};


#[test]
fn test_return_data_non_nesting() {
    let instruction_caller: Instruction = Instruction {
        program_id: Pubkey([0u8;32]),
        utxos: vec![0],
        data: [1u8;32].to_vec()
    };

    let message : Message = Message { signers: vec![], instructions: vec![instruction_caller] };

    let utxo_a = UtxoSharedData::create(vec![1,1,1,1], Pubkey([1;32]), [2;32], 0);

    let mut tnx_utxos = Vec::new();
    tnx_utxos.push((utxo_a.utxo_id(),utxo_a));

    let mut transaction_context : TransactionContext = TransactionContext::new(tnx_utxos, 4, 20);

    let mut file = File::open("./compiled-ebpf/return-data.so").expect("can't read the elf file");

    let mut caller = Vec::new();
    file.read_to_end(&mut caller).unwrap();

    let mut programs: HashMap<String,Vec<u8>> = HashMap::new();
    programs.insert(digest(digest(Pubkey([0u8;32]).as_ref())), caller);

    let _ = MessageProcessor::process_message(message, &mut transaction_context, programs).expect("Failed message processing");

    println!("{:?}",transaction_context.get_return_data());
}