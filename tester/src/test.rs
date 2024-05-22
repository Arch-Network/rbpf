use std::{collections::HashMap, fs::File, io::Read, sync::Arc};

use solana_rbpf::{
    aligned_memory::AlignedMemory, ebpf, elf::Executable, memory_region::{MemoryMapping, MemoryRegion}, program::{BuiltinProgram, FunctionRegistry}, verifier::RequisiteVerifier, vm::{Config, EbpfVm, TestContextObject}
};
use crate::types::*;

pub fn test_everything() {
    println!("Hello");
    let mut file = File::open("ebpf.so").unwrap();
    let mut elf = Vec::new();
    file.read_to_end(&mut elf).unwrap();
    let executable =
        Executable::<TestContextObject>::from_elf(&elf, Arc::new(BuiltinProgram::new_mock()))
            .unwrap();
    // vm configuration
    let config = Config::default();

    let program = executable.get_text_bytes().1;
        // provides context about runtime (ex: number of instruction left)
        let mut context_object: TestContextObject = TestContextObject::new(1500);

        // verifier for bpf
        executable.verify::<RequisiteVerifier>().unwrap();
    
        let sbpf_version = executable.get_sbpf_version();
    
        let mut stack =
            AlignedMemory::<{ ebpf::HOST_ALIGN }>::zero_filled(executable.get_config().stack_size());
        let stack_len = stack.len();
        println!("stack {stack_len}");
    
        let mut heap = AlignedMemory::<{ ebpf::HOST_ALIGN }>::with_capacity(12 * 1024);
    
        let mem_region = MemoryRegion::new_writable(&mut construct_data(), ebpf::MM_INPUT_START);
    
        let regions: Vec<MemoryRegion> = vec![
            executable.get_ro_region(),
            MemoryRegion::new_writable(stack.as_slice_mut(), ebpf::MM_STACK_START),
            MemoryRegion::new_writable(heap.as_slice_mut(), ebpf::MM_HEAP_START),
            mem_region,
        ];
    println!("Memory Region");
        let memory_mapping =
            MemoryMapping::new(regions, executable.get_config(), sbpf_version).unwrap();
    
        let mut vm = EbpfVm::new(
            executable.get_loader().clone(),
            sbpf_version,
            &mut context_object,
            memory_mapping,
            stack_len,
        );
    
        let (instruction_count, result) = vm.execute_program(&executable, true);
        println!("result is {:?}", result.unwrap());
    
    
}

fn construct_data() ->  Vec<u8> {
    let utxo = UtxoMeta{ txid: String::from("a"), vout: 2 };
    let instruction = Instruction {
        program_id: Pubkey([1u8;32].to_vec()),
        utxos: vec![utxo.clone()],
        data: [12u8;1024].to_vec(),
    };
    let mut authority = HashMap::new();
    authority.insert(String::from("a:2"), [12u8;78].to_vec());

    let mut data: HashMap<String, Vec<u8>> = HashMap::new();
    data.insert(String::from("a:2"), [12u8;78].to_vec());

    let input_to_entrypoint = VmInput {
        instruction, 
        authority,
        data
    };
    let mut serialised_data = borsh::to_vec(&input_to_entrypoint).unwrap();
    let mut data_len = serialised_data.len() as u32;
    let mut data_len = data_len.to_le_bytes().to_vec();
    
    data_len.append(&mut serialised_data);
    data_len
}