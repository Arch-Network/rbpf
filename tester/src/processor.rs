use core::fmt;
use std::{alloc::Layout, cell::RefCell, collections::HashMap, env::current_exe, rc::Rc, sync::Arc};

use core_types::types::{Instruction, Pubkey, Transaction, UtxoInfo, UtxoMeta};
use sha256::digest;
use solana_rbpf::{aligned_memory::AlignedMemory, ebpf::{self, MM_HEAP_START}, elf::Executable, memory_region::{MemoryMapping, MemoryRegion}, verifier::RequisiteVerifier, vm::{ContextObject, EbpfVm}};

use crate::config::create_program_runtime_environment_v1;

pub const MAX_COMPUTE_VALUE:u64 =  15000000000;

pub struct MessageProcessor {}

impl MessageProcessor {
    pub fn process_message(
        message : Message,
        transaction_context : &mut TransactionContext,
        log_collector: Option<Rc<RefCell<LogCollector>>>,
        programs : HashMap<String,Vec<u8>>,
    ) {

        let traces = vec![];
        let mut invoke_context = InvokeContext::new(
            transaction_context,
            log_collector,
            programs,
            RefCell::new(MAX_COMPUTE_VALUE),
            traces,
        );

        // this is processing of a transaction
        for instruction in message.instructions {
            invoke_context.process_instruction(instruction);
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AllocErr;
impl fmt::Display for AllocErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Error: Memory allocation failed")
    }
}

pub struct BpfAllocator {
    len: u64,
    pos: u64,
}

impl BpfAllocator {
    pub fn new(len: u64) -> Self {
        Self { len, pos: 0 }
    }

    pub fn alloc(&mut self, layout: Layout) -> Result<u64, AllocErr> {
        let bytes_to_align = (self.pos as *const u8).align_offset(layout.align()) as u64;
        if self
            .pos
            .saturating_add(bytes_to_align)
            .saturating_add(layout.size() as u64)
            <= self.len
        {
            self.pos = self.pos.saturating_add(bytes_to_align);
            let addr = MM_HEAP_START.saturating_add(self.pos);
            self.pos = self.pos.saturating_add(layout.size() as u64);
            Ok(addr)
        } else {
            Err(AllocErr)
        }
    }
}

pub struct SyscallContext {
    pub allocator: BpfAllocator,
    pub trace_log: Vec<[u64; 12]>,
}

pub struct InvokeContext<'a> {
    transaction_context : &'a mut TransactionContext,
    log_collector: Option<Rc<RefCell<LogCollector>>>,
    programs : HashMap<String,Vec<u8>>,
    compute_meter: RefCell<u64>,
    traces: Vec<[u64; 12]>,
    pub syscall_context: Vec<Option<SyscallContext>>,
}

impl<'a> ContextObject for InvokeContext<'a> {
    fn trace(&mut self, state: [u64; 12]) {
        self.traces.push(state);
    }

    fn consume(&mut self, amount: u64) {
        // 1 to 1 instruction to compute unit mapping
        // ignore overflow, Ebpf will bail if exceeded
        let mut compute_meter = self.compute_meter.borrow_mut();
        *compute_meter = compute_meter.saturating_sub(amount);
    }

    fn get_remaining(&self) -> u64 {
        *self.compute_meter.borrow()
    }
}

impl<'a> InvokeContext<'a> {
    pub fn new(
        transaction_context : &'a mut TransactionContext,
        log_collector: Option<Rc<RefCell<LogCollector>>>,
        programs : HashMap<String,Vec<u8>>,
        compute_meter: RefCell<u64>,
        traces: Vec<[u64; 12]>,
    ) -> Self {
        Self {
            transaction_context,
            log_collector,
            programs,
            compute_meter,
            traces,
            syscall_context: Vec::new(),
        }
    }
    pub fn process_instruction(
        &mut self,
        instruction : Instruction
    ) -> Result<(), String> {
        self.transaction_context
            .get_next_instruction_context()
            .configure(instruction);
        self.push()?;
        self.process_executable_chain()
            // MUST pop if and only if `push` succeeded, independent of `result`.
            // Thus, the `.and()` instead of an `.and_then()`.
            .and(self.pop())
    }

    pub fn get_syscall_context_mut(&mut self) -> Result<&mut SyscallContext, String> {
        self.syscall_context
            .last_mut()
            .and_then(|syscall_context| syscall_context.as_mut())
            .ok_or("call Depth error".into())
    }

    pub fn pop(&mut self) -> Result<(), String> {
        self.transaction_context.pop()
    }

    pub fn push(&mut self) -> Result<(),String> {
        let instruction_context = self
        .transaction_context
        .get_instruction_context_at_index_in_trace(
            self.transaction_context.get_instruction_trace_length(),
        );
        // TODO : check reentrancy later

        self.transaction_context.push()?;
        Ok(())
    }

    fn process_executable_chain(
        &mut self,
    ) -> Result<(), String> {
        let mut mem = serealise(&self.transaction_context);
        let current_ins_context = self.transaction_context.get_current_instruction_context();
        // Part One: Transaction Procesing
        
        // elf file
        let elf = self.programs.get(&digest(digest(current_ins_context.instruction.program_id.0.clone()))).expect("can't find the key associated with the program account");

        let mut result = create_program_runtime_environment_v1(false);

        let executable =
        Executable::<InvokeContext>::from_elf(&elf, Arc::new(result.unwrap())).unwrap();

    let program = executable.get_text_bytes().1;
    let executable_registry = executable.get_function_registry();
        let loader_registry = executable.get_loader().get_function_registry();

        // println!("executable {:?}\n\nloader {:?}\n\n", executable_registry,loader_registry);

        // verifier for bpf
        executable.verify::<RequisiteVerifier>().unwrap();
    
        let sbpf_version = executable.get_sbpf_version();
        // println!(" version {:?}",sbpf_version);
    
        let mut stack =
            AlignedMemory::<{ ebpf::HOST_ALIGN }>::zero_filled(executable.get_config().stack_size());
        let stack_len = stack.len();
    
        let mut heap = AlignedMemory::<{ ebpf::HOST_ALIGN }>::zero_filled(120 * 1024);
    
        mem.extend_from_slice(&[0u8;1024]);
        let mem_region = MemoryRegion::new_writable(&mut mem, ebpf::MM_INPUT_START);
    
        let regions: Vec<MemoryRegion> = vec![
            executable.get_ro_region(),
            MemoryRegion::new_writable_gapped(stack.as_slice_mut(), ebpf::MM_STACK_START, 0),
            MemoryRegion::new_writable(heap.as_slice_mut(), ebpf::MM_HEAP_START),
            mem_region,
        ];


        let memory_mapping =
            MemoryMapping::new(regions, executable.get_config(), sbpf_version).unwrap();
    
        let mut vm: EbpfVm<InvokeContext> = EbpfVm::new(
            executable.get_loader().clone(),
            sbpf_version,
            self,
            memory_mapping,
            stack_len,
        );

        // PART TWO : POST PROCESSING
        let transaction = deserialise(&mut self.transaction_context, mem);

        // now all the authorities have been updated inside the transaction contexts' structs
        // TODO: update authorities in main database

        Ok(())
    }


}


#[derive(Debug, Clone)]
pub struct Message {
    pub signers: Vec<Pubkey>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub struct TransactionContext {
    // instructions: Vec<Instruction>,
    instruction_stack_capacity: usize,
    instruction_trace_capacity: usize,
    instruction_stack: Vec<usize>,
    instruction_trace: Vec<InstructionContext>,
    authorities : HashMap<String,Vec<u8>>,
    data : HashMap<String,Vec<u8>>
}

impl TransactionContext {
    pub fn new(
        // instructions : Vec<Instruction>,
        instruction_stack_capacity: usize,
        instruction_trace_capacity: usize,
        authorities : HashMap<String,Vec<u8>>,
        data : HashMap<String,Vec<u8>>
    ) -> Self {
            Self {
                // instructions,
                instruction_stack_capacity,
                instruction_trace_capacity,
                instruction_stack: Vec::with_capacity(instruction_stack_capacity),
                instruction_trace: vec![InstructionContext::default()],
                authorities,
                data
            }
    }

    pub fn get_authorities(&self ) -> &HashMap<String,Vec<u8>> {
        &self.authorities
    }

    pub fn get_data(&self ) -> &HashMap<String,Vec<u8>> {
        &self.data
    }

    pub fn push(&mut self) -> Result<(), String> {
        let nesting_level = self.get_instruction_context_stack_height();

        let instruction_context = self.get_next_instruction_context();
        instruction_context.nesting_level = nesting_level;
            
        let index_in_trace = self.get_instruction_trace_length();
        if index_in_trace >= self.instruction_trace_capacity {
            return Err("MaxInstructionTraceLengthExceeded".into());
        }

        self.instruction_trace.push(InstructionContext::default());

        if nesting_level >= self.instruction_stack_capacity {
            return Err("Call Depth Error".into());
        }
        self.instruction_stack.push(index_in_trace);

        Ok(())
    }

    pub fn pop(&mut self) -> Result<(), String>  {
        if self.instruction_stack.is_empty() {
            return Err("Call Depth Error".into());
        }

        if let Some(v) = self.instruction_stack.pop() {
            return Ok(())
        } else {
            return Err("Error in removing instruction context".into())
        }

    }

    pub fn update_utxo_authority(&mut self, id: &String,value : &Vec<u8>) -> Result<(), String> {
        let mut authority = self.authorities.get_mut(id).ok_or::<String>("no matching authority found".into())?;
        *authority = (*value.clone()).to_vec();

        Ok(())
    }

    pub fn update_utxo_data(&mut self, id: &String,value : &Vec<u8>) -> Result<(), String> {
        let mut authority = self.data.get_mut(id).ok_or::<String>("no matching data found".into())?;
        *authority = (*value.clone()).to_vec();

        Ok(())
    }
    pub fn get_instruction_context_stack_height(&self) -> usize {
        self.instruction_stack.len()
    }

    pub fn get_instruction_context_at_nesting_level(&self, nesting_level : usize) -> &InstructionContext {
        let index_in_trace = *self
            .instruction_stack
            .get(nesting_level)
            .unwrap();

        self.get_instruction_context_at_index_in_trace(index_in_trace).unwrap()
    }

    pub fn get_instruction_trace_length(&self) -> usize {
        self.instruction_trace.len().saturating_sub(1)
    }

    pub fn get_instruction_context_at_index_in_trace(
        &self,
        index_in_trace: usize,
    ) -> Result<&InstructionContext,String> { 
       self.instruction_trace
            .get(index_in_trace).ok_or("context doesn't exists, can't access instruction context at this level".into())
            
    }

    pub fn get_next_instruction_context(
        &mut self,
    ) -> &mut InstructionContext {
        self.instruction_trace
            .last_mut().unwrap()
    }

    pub fn get_current_instruction_context(&self) -> &InstructionContext {
        let level = self
            .get_instruction_context_stack_height()
            .checked_sub(1)
            .unwrap();

        self.get_instruction_context_at_nesting_level(level)
    }
       
}

#[derive(Debug, Clone, Default)]
pub struct InstructionContext {
    nesting_level : usize,
    instruction: Instruction
}

impl InstructionContext {
    pub fn configure(&mut self, instruction : Instruction)  {
        self.instruction = instruction
    }
}

fn serealise(transaction_context : &TransactionContext) -> Vec<u8>{
    let current_context = transaction_context.get_current_instruction_context();
    let (utxo_authorities, utxo_data) = (&transaction_context.authorities,&transaction_context.data);

    let instruction = &current_context.instruction;
    let mut serialised_data = borsh::to_vec(&(instruction,utxo_authorities,utxo_data)).unwrap();
    let mut data_len = serialised_data.len() as u32;
    let mut data_len = data_len.to_le_bytes().to_vec();
    
    data_len.append(&mut serialised_data);
    data_len
}

fn deserialise(transaction_context : &mut TransactionContext, mem : Vec<u8>) -> Transaction {
    let instruction_context = transaction_context.get_current_instruction_context();
    let current_program_id = &instruction_context.instruction.program_id.clone();

    let length = [mem[0],mem[1],mem[2],mem[3]];
    let length_of_output = u32::from_le_bytes(length);

    let (mut output_utxos, transaction)  = borsh::from_slice::<(Vec<UtxoInfo>,Transaction)>(&mem[4..(length_of_output + 4) as usize]).expect("can't deser");

    // update authorities after checking if program could update the authorities
    for output_utxo in output_utxos.iter_mut() {
        let output_utox_id = output_utxo.id();
        let utxo_authority_in_context = transaction_context.authorities.get(&output_utox_id).expect("must have a key in authority");
        if *utxo_authority_in_context == current_program_id.0 {

            let updated_owner =output_utxo.authority.get_mut();

            // checking here can save us updating hashmap cost
            if  updated_owner.0!= *transaction_context.authorities.get(&output_utox_id).unwrap() {
                let _ = transaction_context.update_utxo_authority(&output_utox_id,&updated_owner.0);
            }
            let _ =transaction_context.update_utxo_data(&output_utox_id,&output_utxo.data.get_mut());
       }
    }
    transaction

}

pub struct LogCollector {
    messages: Vec<String>,
    bytes_written: usize,
    bytes_limit: Option<usize>,
    limit_warning: bool,
}

const LOG_MESSAGES_BYTES_LIMIT: usize = 10 * 1000;

impl Default for LogCollector {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            bytes_written: 0,
            bytes_limit: Some(LOG_MESSAGES_BYTES_LIMIT),
            limit_warning: false,
        }
    }
}
// pub struct InvokeContext<'a> {
//     pub transaction_context: &'a mut TransactionContext,
//     log_collector: Option<Rc<RefCell<LogCollector>>>,
//     traces: Vec<Vec<[u64; 12]>>,
// }

// #[derive(Debug, Clone)]
// pub struct TransactionContext {
//     utxos: Rc<TransactionUTXOS>,
//     instruction_stack_capacity: usize,
//     instruction_trace_capacity: usize,
//     instruction_stack: Vec<usize>,
//     instruction_trace: Vec<InstructionContext>,
//     return_data: TransactionReturnData,
// }

// impl TransactionContext {
//     pub fn new(
//         transaction_utxos: Vec<TransactionUTXO>,
//         instruction_stack_capacity: usize,
//         instruction_trace_capacity: usize,
//     ) -> Self  {
//         Self {
//             utxos: transaction_utxos,
//             instruction_stack_capacity,
//             instruction_trace_capacity,
//             instruction_stack: todo!(),
//             instruction_trace: vec![InstructionContext::default()],
//             return_data: TransactionReturnData::default(),
//         }
//     }
// }

// #[derive(Debug, Clone)]
// pub struct InstructionContext {
//     nesting_level: usize,
//     program_utxos: Vec<Pubkey>,
//     instruction_utxos: Vec<InstructionUTXO>,
//     instruction_data: Vec<u8>,
// }

// #[derive(Debug, Clone)]
// pub struct TransactionReturnData {
//     pub program_id: Pubkey,
//     pub data: Vec<u8>
// }

// #[derive(Debug, Clone)]
// pub struct TransactionUTXOS {
//     accounts: Vec<RefCell<UTXOSharedData>>,
//     touched_flags: RefCell<Box<[bool]>>,
// }

// #[derive(Debug, Clone)]
// pub struct UTXOSharedData {
//     authority : Pubkey,
//     data: Vec<u8>,
// }