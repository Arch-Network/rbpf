use std::{fs::File, io::Read, sync::Arc};

use solana_rbpf::{assembler::assemble, declare_builtin_function, ebpf, elf::Executable, error::EbpfError, memory_region::{AccessType, MemoryMapping}, program::{BuiltinFunction, BuiltinProgram, SBPFVersion}, vm::TestContextObject};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

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

    println!("Executable : {:?}", executable);
    let program = executable.get_text_bytes().1;
    println!("{:x?}", program);

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
    let mut mem = unsafe {string_to_write.as_bytes_mut()};
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

    println!("{:?}",vm);
    let (instruction_count, result) = vm.execute_program(&executable, true);
    println!("result is {:?}",result.unwrap());

    println!("The memory region after reading {:?}",mem);

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
