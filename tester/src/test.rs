use std::{collections::HashMap, fs::File, io::Read, sync::Arc};
use core_types::types::*;
use solana_rbpf::{
    aligned_memory::AlignedMemory, ebpf, elf::Executable, memory_region::{MemoryMapping, MemoryRegion}, program::{BuiltinProgram, FunctionRegistry}, verifier::RequisiteVerifier, vm::{Config, EbpfVm, TestContextObject}
};
use core_types::types::*;

pub fn test_everything() {
    // let mut file = File::open("src/ebpf.so").unwrap();
    let mut file = File::open("../target/sbf-solana-solana/release/ebpf.so").unwrap();

    // let mut file = File::open("src/ebpf-old.so").unwrap();
    let mut elf = Vec::new();
    file.read_to_end(&mut elf).unwrap();

    // let config = Config {
    //     max_call_depth: 20,
    //     stack_frame_size: 4096,
    //     enable_address_translation: true, // To be deactivated once we have BTF inference and verification
    //     enable_stack_frame_gaps: false,
    //     instruction_meter_checkpoint_distance: 10000,
    //     enable_instruction_meter: true,
    //     enable_instruction_tracing: true,
    //     enable_symbol_and_section_labels: true,
    //     reject_broken_elfs: false,
    //     noop_instruction_rate: 256,
    //     sanitize_user_provided_values: true,
    //     external_internal_function_hash_collision: true,
    //     reject_callx_r10: true,
    //     enable_sbpf_v1: true,
    //     enable_sbpf_v2: false,
    //     optimize_rodata: true,
    //     new_elf_parser: true,
    //     aligned_memory_mapping: true,
    //     // Warning, do not use `Config::default()` so that configuration here is explicit.
    // BuiltinProgram::new_loader(config, FunctionRegistry::default())
    // };
    let executable =
        Executable::<TestContextObject>::from_elf(&elf, Arc::new(BuiltinProgram::new_mock()))
            .unwrap();

    let program = executable.get_text_bytes().1;
        // provides context about runtime (ex: number of instruction left)
        let mut context_object: TestContextObject = TestContextObject::new(15000000000);

        // The Murmur3 hash value (used by RBPF) of the string "entrypoint"
        // const ENTRYPOINT_KEY: u32 = 0x71E3CF81;

        // let function = executable.get_loader()
        //         .get_function_registry()
        //         .lookup_by_key(ENTRYPOINT_KEY)
        //         .map(|(_name, function)| function).expect("Should be present");

        // println!("func {:?}",function);

        let executable_registry = executable.get_function_registry();
        let loader_registry = executable.get_loader().get_function_registry();

        println!("executable {:?}\n\nloader {:?}\n\n", executable_registry,loader_registry);

        // verifier for bpf
        executable.verify::<RequisiteVerifier>().unwrap();
    
        let sbpf_version = executable.get_sbpf_version();
        // println!(" version {:?}",sbpf_version);
    
        let mut stack =
            AlignedMemory::<{ ebpf::HOST_ALIGN }>::zero_filled(executable.get_config().stack_size());
        let stack_len = stack.len();
    
        let mut heap = AlignedMemory::<{ ebpf::HOST_ALIGN }>::zero_filled(120 * 1024);
    
        let mut mem = construct_data();
        mem.extend_from_slice(&[0u8;1024]);
        let mem_region = MemoryRegion::new_writable(&mut mem, ebpf::MM_INPUT_START);
    
        let regions: Vec<MemoryRegion> = vec![
            executable.get_ro_region(),
            MemoryRegion::new_writable_gapped(stack.as_slice_mut(), ebpf::MM_STACK_START, 0),
            MemoryRegion::new_writable(heap.as_slice_mut(), ebpf::MM_HEAP_START),
            mem_region,
        ];
    println!("Memory Region");
        let memory_mapping =
            MemoryMapping::new(regions, executable.get_config(), sbpf_version).unwrap();
    
        let mut vm: EbpfVm<TestContextObject> = EbpfVm::new(
            executable.get_loader().clone(),
            sbpf_version,
            &mut context_object,
            memory_mapping,
            stack_len,
        );
        // EbpfVm::invoke_function(&mut vm, function);
    
        let (instruction_count, result) = vm.execute_program(&executable, true);
        println!("result is {:?}", result);
        // println!("{:?}",mem);
    
    
}

fn construct_data() ->  Vec<u8> {
    // let utxo = UtxoMeta{ txid: String::from("a"), vout: 2 };
    // let instruction = Instruction {
    //     program_id: Pubkey([1u8;32].to_vec()),
    //     utxos: vec![utxo.clone()],
    //     data: [12u8;1024].to_vec(),
    // };
    // let mut authority = HashMap::new();
    // authority.insert(String::from("a:2"), [12u8;78].to_vec());

    // let mut data: HashMap<String, Vec<u8>> = HashMap::new();
    // data.insert(String::from("a:2"), [12u8;78].to_vec());

    // let input_to_entrypoint = VmInput {
    //     instruction, 
    //     authority,
    //     data
    // };
    // let mut serialised_data = borsh::to_vec(&input_to_entrypoint).unwrap();
    // let mut data_len = serialised_data.len() as u32;
    // let mut data_len = data_len.to_le_bytes().to_vec();
    
    // data_len.append(&mut serialised_data);
    // data_len
    
    let ins = ProgramInstruction::Init(String::from("abcdef"));
    let mut serialised = borsh::to_vec(&ins).expect("unable to serialise string");
    let size = serialised.len();
    let mut bytes= size.to_le_bytes().to_vec();
    bytes.append(&mut serialised);
    bytes
    // vec![0,0,0,4]
}