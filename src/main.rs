use std::{fs::File, io::Read, mem::size_of, slice::from_raw_parts_mut, sync::Arc};
mod serialise;
mod consts;
mod types;
mod error;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use solana_rbpf::{
    assembler::assemble,
    declare_builtin_function, ebpf,
    elf::Executable,
    error::EbpfError,
    insn_builder::Instruction,
    memory_region::{AccessType, MemoryMapping},
    program::{BuiltinFunction, BuiltinProgram, SBPFVersion},
    vm::TestContextObject,
};

use std::{slice::from_raw_parts, str::from_utf8};
type Error = Box<dyn std::error::Error>;
fn main() {
    use solana_rbpf::{
        aligned_memory::AlignedMemory,
        ebpf,
        memory_region::MemoryRegion,
        program::{BuiltinProgram, FunctionRegistry},
        verifier::RequisiteVerifier,
        vm::{Config, EbpfVm},
    };

    // let code = "
    // mov64 r2, 0x14
    // syscall bpf_syscall_get_str
    // exit
    // ";

    let mut file = File::open("relative_call.so").unwrap();
    let mut elf = Vec::new();
    file.read_to_end(&mut elf).unwrap();
    let executable =
        Executable::<TestContextObject>::from_elf(&elf, Arc::new(BuiltinProgram::new_mock()))
            .unwrap();
    // vm configuration
    let config = Config::default();

    // Function Symbol mapping
    // let mut function_registry = FunctionRegistry::<BuiltinFunction<TestContextObject>>::default();

    // add our custom fucntion to registry so that function symbols can be populated
    // function_registry
    //     .register_function_hashed("bpf_syscall_get_str".as_bytes(), SyscallGetStr::vm)
    //     .expect("should be able to add custom fucntion to registry");

    // loader of the executable
    // let loader = Arc::new(BuiltinProgram::new_loader(config, function_registry));

    // elf
    // let executable = assemble(code, loader.clone()).expect("Shouled be able to assemble");

    // println!("Executable : {:?}", executable);
    let program = executable.get_text_bytes().1;
    // println!("{:x?}", program);

    // provides context about runtime (ex: number of instruction left)
    let mut context_object: TestContextObject = TestContextObject::new(1500);

    // verifier for bpf
    executable.verify::<RequisiteVerifier>().unwrap();

    let sbpf_version = executable.get_sbpf_version();

    let mut stack =
        AlignedMemory::<{ ebpf::HOST_ALIGN }>::zero_filled(executable.get_config().stack_size());
    let stack_len = stack.len();
    println!("stack {stack_len}");

    let mut heap = AlignedMemory::<{ ebpf::HOST_ALIGN }>::with_capacity(0);

    let mut string_to_write = String::from("Hello");
    let mut mem = unsafe { string_to_write.as_bytes_mut() };
    let mem_region = MemoryRegion::new_writable(&mut mem, ebpf::MM_INPUT_START);

    let regions: Vec<MemoryRegion> = vec![
        executable.get_ro_region(),
        MemoryRegion::new_writable(stack.as_slice_mut(), ebpf::MM_STACK_START),
        MemoryRegion::new_writable(heap.as_slice_mut(), ebpf::MM_HEAP_START),
        mem_region,
    ];

    let memory_mapping =
        MemoryMapping::new(regions, executable.get_config(), sbpf_version).unwrap();

    let mut vm = EbpfVm::new(
        executable.get_loader().clone(),
        sbpf_version,
        &mut context_object,
        memory_mapping,
        stack_len,
    );

    println!("{:?}", vm);
    let (instruction_count, result) = vm.execute_program(&executable, true);
    println!("result is {:?}", result.unwrap());

    println!("The memory region after reading {:?}", mem);

    // const START: u64 = 0x100000000;
    //     const LENGTH: u64 = 1000;

    //     let data = vec![0u8; LENGTH as usize];
    //     let addr = data.as_ptr() as u64;
    //     let config = Config::default();
    //     let memory_mapping = MemoryMapping::new(
    //         vec![MemoryRegion::new_readonly(&data, START)],
    //         &config,
    //         &SBPFVersion::V2,
    //     )
    //     .unwrap();

    // let cases = vec![
    //     (true, START, 0, addr),
    //     (true, START, 1, addr),
    //     (true, START, LENGTH, addr),
    //     (true, START + 1, LENGTH - 1, addr + 1),
    //     (false, START + 1, LENGTH, 0),
    //     (true, START + LENGTH - 1, 1, addr + LENGTH - 1),
    //     (true, START + LENGTH, 0, addr + LENGTH),
    //     (false, START + LENGTH, 1, 0),
    //     (false, START, LENGTH + 1, 0),
    //     (false, 0, 0, 0),
    //     (false, 0, 1, 0),
    //     (false, START - 1, 0, 0),
    //     (false, START - 1, 1, 0),
    //     (true, START + LENGTH / 2, LENGTH / 2, addr + LENGTH / 2),
    // ];
    // for (ok, start, length, value) in cases {
    //     if ok {
    //         println!("start: {}, length: {}, value: {}, is_correct: {}\n",start,length,value,ok);
    //         assert_eq!(
    //             translate(&memory_mapping, AccessType::Load, start, length).unwrap(),
    //             value
    //         )
    //     } else {
    //         assert!(translate(&memory_mapping, AccessType::Load, start, length).is_err())
    //     }
    // }
}
// fn translate(
// //     memory_mapping: &MemoryMapping,
// //     access_type: AccessType,
// //     vm_addr: u64,
// //     len: u64,
// // ) -> Result<u64, Error> {
// //     memory_mapping
// //         .map(access_type, vm_addr, len)
// //         .map_err(|err| err.into())
// //         .into()
// // }

declare_builtin_function!(
    /// Prints a NULL-terminated UTF-8 string.
    SyscallGetStr,
    fn rust(
        _context_object: &mut TestContextObject,
        vm_addr: u64,
        len: u64,
        _arg3: u64,
        _arg4: u64,
        _arg5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        println!("vm_addr:{:x}, len: {:x}",vm_addr,len);
        let host_addr: Result<u64, EbpfError> =
            memory_mapping.map(AccessType::Load, vm_addr, len).into();
        let host_addr = host_addr?;
        println!("host_addr : {:x?}",host_addr);
        return Ok(host_addr);
    //     let mut message;
    //     unsafe {
    //         message = from_utf8(from_raw_parts(host_addr as *const u8, len as usize))
    //             .unwrap_or("Invalid UTF-8 String");
    //         println!("log: {message}");
    //     }

    //    if let Ok(num) = message.as_bytes().read_u64::<BigEndian>() {
    //     return Ok(num);
    //    } else {
    //     return Err("Length of string too big".into())
    //    }

    }
);

// fn asm(src: &str) -> Result<Vec<ebpf::Insn>, String> {
//     let executable = assemble::<TestContextObject>(src, Arc::new(BuiltinProgram::new_mock()))?;
//     let (_program_vm_addr, program) = executable.get_text_bytes();
//     Ok((0..program.len() / ebpf::INSN_SIZE)
//         .map(|insn_ptr| ebpf::get_insn(program, insn_ptr))
//         .collect())
// }

mod test {
    use std::{
        mem::{align_of, size_of, size_of_val},
        slice::from_raw_parts,
    };

    use crate::{deserialize, serealize, Pubkey, UtxoInfo};

    // #[test]
    // fn abc() {
    //     let vec = vec![9u32, 10,11,12,13,14,15,16,17,18];
    //     let vec_pointer = vec.as_ptr();

    //    let a =  unsafe{
    //     from_raw_parts(vec_pointer, 4)
    // };

    #[test]
    // fn check_ser() {
    //     let pubkey = Pubkey::from_array([123;32]);
    //     let utxo1 = UtxoInfo::random();
    //     let utxo2 = UtxoInfo::from_data(String::from("ABCD"),45,44);
    //     let utxos = [utxo1,utxo2];
    //     let ins_data = [0_u8,1,2,3,4,5,6,77];
    //     let mut data = serealize(pubkey, &utxos, ins_data.as_slice());

    // println!("{data:?}");
    // // let mut data = [0,0,0,1];
    //     let input = data.as_mut_ptr();
    //     let (pubkey,utxos,instructions) = unsafe {deserialize(input)};

    //     // println!("pubkey {:?}, utxos {:?}, instructions: {:?}",pubkey,utxos,instructions);

    // }
    #[test]
    fn check_alignment() {
        const ALIGN: usize = 32768;
        fn get_mem_zeroed(max_len: usize) -> (Vec<u8>, usize) {
            // use calloc() to get zeroed memory from the OS instead of using
            // malloc() + memset(), see
            // https://github.com/rust-lang/rust/issues/54628
            let mut mem = vec![0; max_len];
            println!("size {:?}", size_of::<Vec<u64>>());
            // println!("mem is {mem:?}");
            let align_offset = mem.as_ptr().align_offset(ALIGN);
            println!("offset is {align_offset:?}");
            mem.resize(max_len.saturating_add(align_offset), 0);
            // println!("{:?}",mem);
            (mem, align_offset)
        }
        // get_mem_zeroed(250);
        // get_mem_zeroed(17);

        //         let x = vec![0_u8,1,2,3,4,5,6];
        //         println!("ptr {:?}",x.as_ptr());
        //         println!("size {:?}",size_of_val(&x));
        // let ptr = x.as_ptr();
        // let offset = ptr.align_offset(align_of::<u32>());
        // println!("align of u32 {:?}",align_of::<u32>() );
        // println!("{:?}",offset);

        fn get_mem(max_len: usize) -> (Vec<u8>, usize) {
            let mut mem: Vec<u8> = Vec::with_capacity(max_len.saturating_add(ALIGN));
            println!("mem {:?}", mem);
            mem.push(0);
            println!("mem {:?}", mem);
            let align_offset = mem.as_ptr().align_offset(ALIGN);
            println!("offset is {align_offset:?}");
            mem.resize(align_offset, 0);
            println!("retured mem {:?}", mem);
            (mem, align_offset)
        }

        fn get_info(a: &Vec<u8>) {
            println!(
                "length : {:?}, capacity: {:?}, pointer: {:?}\n",
                a.len(),
                a.capacity(),
                a.as_ptr()
            );
            println!("elements {:?}\n", a);
        }
        let mut a = Vec::with_capacity(4);
        a.extend_from_slice(&[1, 2, 3, 4_u8]);
        get_info(&a);
        let align_offset = a.as_ptr().align_offset(16);
        println!("offset is {align_offset:?}");

        a.resize(5, 0);
        get_info(&a);

        // get_mem(15);
    }

    // if offset < x.len() - 1 {
    //     let u16_ptr = ptr.add(offset).cast::<u16>();
    //     assert!(*u16_ptr == u16::from_ne_bytes([5, 6]) || *u16_ptr == u16::from_ne_bytes([6, 7]));
    // } else {
    //     // while the pointer can be aligned via `offset`, it would point
    //     // outside the allocation

    //     }
}

fn serealize(pubkey: Pubkey, utxos: &[UtxoInfo], ins_data: &[u8]) -> Vec<u8> {
    let mut vec = Vec::new();
    let num_utxos = utxos.len() as u64;
    vec.extend_from_slice(num_utxos.to_le_bytes().as_slice());

    for i in 0..num_utxos as usize {
        let utxo = &utxos[i];
        let txid = utxo.txid;
        vec.extend_from_slice(txid.as_slice());

        let vout = utxo.vout;
        vec.extend_from_slice(vout.to_le_bytes().as_slice());

        let value = utxo.value;
        vec.extend_from_slice(value.to_le_bytes().as_slice());
    }
    let instruction_len = ins_data.len() as u64;
    vec.extend_from_slice(instruction_len.to_le_bytes().as_slice());

    vec.extend_from_slice(ins_data);

    vec.extend_from_slice(pubkey.into_array().as_slice());
    vec
}

pub unsafe fn deserialize<'a>(input: *mut u8) -> (&'a Pubkey, Vec<UtxoInfo>, &'a [u8]) {
    let mut offset: usize = 0;
    println!("mut *u8 {:?}", unsafe { *(input as *const u64) });
    let num_utxos = *(input.add(offset) as *const u64) as usize;
    offset += size_of::<u64>();
    println!("num_utxo {:?}\n", num_utxos);

    // Account Infos
    let mut utxos = Vec::with_capacity(num_utxos);
    println!("Vec initlized");
    for _ in 0..num_utxos {
        let txid = *(input.add(offset) as *const [u8; 32]);
        offset += size_of::<[u8; 32]>();

        println!("txid {:?}\n", txid);
        let vout = *(input.add(offset) as *const u32);
        offset += size_of::<u32>();
        println!("vout {:?}\n", vout);

        let value = *(input.add(offset) as *const u64);
        offset += size_of::<u64>();

        let utxo = UtxoInfo::from_data(txid, vout, value);
        utxos.push(utxo);
    }

    let instruction_data_len = *(input.add(offset) as *const usize);
    offset += size_of::<u64>();

    let instruction_data = { from_raw_parts(input.add(offset), instruction_data_len) };
    offset += instruction_data_len;
    offset += (offset as *const u8).align_offset(BPF_ALIGN_OF_U128); // padding

    // Program Id
    let program_id: &Pubkey = &*(input.add(offset) as *const Pubkey);

    (program_id, utxos, instruction_data)
}
