use core::fmt;
use std::{alloc::Layout, cell::{RefCell, RefMut}, collections::HashMap, env::current_exe, rc::Rc, sync::Arc};

use core_types::{entrypoint::MAX_PERMITTED_DATA_LENGTH, types::{Instruction, Pubkey, Transaction, UtxoMeta}};
use sha256::digest;
use solana_program::address_lookup_table::instruction;
use solana_rbpf::{aligned_memory::AlignedMemory, ebpf::{self, MM_HEAP_START}, elf::Executable, memory_region::{MemoryMapping, MemoryRegion}, verifier::RequisiteVerifier, vm::{ContextObject, EbpfVm}};

use crate::{config::create_program_runtime_environment_v1, errors::{InstructionError, TransactionError}, serialization::{deserialize_parameters, serialize_parameters}};

pub const MAX_COMPUTE_VALUE:u64 =  15_000_000_000;
pub type IndexOfUtxo = usize;

pub struct MessageProcessor {}

impl MessageProcessor {
    pub fn process_message(
        message : Message,
        transaction_context : &mut TransactionContext,
       /*log_collector: Option<Rc<RefCell<LogCollector>>>,*/
        programs : HashMap<String,Vec<u8>>,
    )  -> Result<(), TransactionError>{

        let traces = vec![];
        let mut invoke_context = InvokeContext::new(
            transaction_context,
            programs,
            RefCell::new(MAX_COMPUTE_VALUE),
            traces,
        );

        // this is processing of a transaction
        for instruction in message.instructions {
            let mut instruction_utxos = Vec::with_capacity(instruction.utxos.len());

            for (instruction_account_index, index_in_transaction) in
                instruction.utxos.iter().enumerate()
            {
                let index_in_callee = instruction
                    .utxos
                    .get(0..instruction_account_index)
                    .ok_or(TransactionError::InvalidAccountIndex)?
                    .iter()
                    .position(|account_index| account_index == index_in_transaction)
                    .unwrap_or(instruction_account_index)
                    as IndexOfUtxo;
                let index_in_transaction = *index_in_transaction as usize;
                instruction_utxos.push(InstructionUtxo {
                    index_in_transaction: index_in_transaction as IndexOfUtxo,
                    index_in_caller: index_in_transaction as IndexOfUtxo,
                    index_in_callee,
                });
            }
            invoke_context.process_instruction(&instruction.data, &instruction_utxos,instruction.program_id.as_ref());
        }

        Ok(())
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
    /*log_collector: Option<Rc<RefCell<LogCollector>>>,*/
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
        programs : HashMap<String,Vec<u8>>,
        compute_meter: RefCell<u64>,
        traces: Vec<[u64; 12]>,
    ) -> Self {
        Self {
            transaction_context,
            programs,
            compute_meter,
            traces,
            syscall_context: Vec::new(),
        }
    }
    pub fn process_instruction(
        &mut self,
        instruction_data: &[u8],
        instruction_utxos: &[InstructionUtxo],
        program_id : &[u8]
    ) -> Result<(), InstructionError> {
        self.transaction_context
            .get_next_instruction_context()
            .configure(instruction_data,instruction_utxos,program_id);
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

    pub fn pop(&mut self) -> Result<(), InstructionError> {
        self.transaction_context.pop()
    }

    pub fn push(&mut self) -> Result<(),InstructionError> {
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
    ) -> Result<(), InstructionError> {

        let (mut parameter_bytes,serialized_accounts) = serialize_parameters(&self.transaction_context,  self.transaction_context.get_current_instruction_context())?;
        // Part One: Transaction Procesing
        
        // elf file
        let elf = self.programs.get(&digest(digest( self.transaction_context.get_current_instruction_context().program_id.as_ref()))).expect("can't find the key associated with the program account");

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
    
        let mem_region = MemoryRegion::new_writable(parameter_bytes.as_slice_mut(), ebpf::MM_INPUT_START);
    
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

        let (instruction_count, result) = vm.execute_program(&executable, true);
        println!("result is {:?}", result);

        // PART TWO : POST PROCESSING
        deserialize_parameters(self.transaction_context,  self.transaction_context.get_current_instruction_context(), parameter_bytes.as_slice(), &serialized_accounts);

        Ok(())
    }


}

#[derive(Debug, Clone)]
pub struct SerializedUtxoMetadata {
    pub original_data_len: usize,
    pub vm_data_addr: u64,
    pub vm_authority_addr: u64,
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
    utxos : Rc<TransactionUtxos>
}

#[derive(Debug, Clone)]
pub struct TransactionUtxos {
    utxos: Vec<RefCell<UtxoSharedData>>,
}

impl TransactionUtxos {
    pub fn from(utxos: Vec<UtxoSharedData> ) -> Self {
    Self {
        utxos : utxos.iter().map(
            |utxo| RefCell::new(utxo.clone())
        ).collect::<Vec<RefCell<UtxoSharedData>>>()
    }
}

    pub fn get(&self, index: IndexOfUtxo) -> Option<&RefCell<UtxoSharedData>> {
        self.utxos.get(index as usize)
    }    
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstructionUtxo {
    /// Points to the utxo and its key in the `TransactionContext`
    pub index_in_transaction: IndexOfUtxo,
    /// Points to the first occurrence in the parent `InstructionContext`
    pub index_in_caller: IndexOfUtxo,
    /// Points to the first occurrence in the current `InstructionContext`
    pub index_in_callee: IndexOfUtxo,
}

#[derive(Debug, Clone)]
pub struct UtxoSharedData {
    /// data held in this utxo
    data: Vec<u8>,
    /// authority over this utxo
    authority: Pubkey,
    /// txid of this utxo
    txid : [u8;32],
    /// vout in btc txn
    vout: u32
}

impl UtxoSharedData {
    pub fn create(
        data: Vec<u8>,
        authority: Pubkey,
        txid: [u8;32],
        vout: u32
    ) -> Self {
        Self { data, authority, txid, vout }
    }
    pub fn get_vout(&self) -> u32 {
        self.vout
    }

    fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }

    fn copy_into_authority_from_slice(&mut self, source: &[u8]) {
        self.authority.as_mut().copy_from_slice(source);
    }
}

impl TransactionContext {
    pub fn new(
        utxos: TransactionUtxos,
        instruction_stack_capacity: usize,
        instruction_trace_capacity: usize,
    ) -> Self {
            Self {
                utxos: Rc::new(utxos),
                instruction_stack_capacity,
                instruction_trace_capacity,
                instruction_stack: Vec::with_capacity(instruction_stack_capacity),
                instruction_trace: vec![InstructionContext::default()],
            }
    }

    pub fn push(&mut self) -> Result<(), InstructionError> {
        let nesting_level = self.get_instruction_context_stack_height();

        let instruction_context = self.get_next_instruction_context();
        instruction_context.nesting_level = nesting_level;
            
        let index_in_trace = self.get_instruction_trace_length();
        if index_in_trace >= self.instruction_trace_capacity {
            return Err(InstructionError::MaxInstructionTraceLengthExceeded);
        }

        self.instruction_trace.push(InstructionContext::default());

        if nesting_level >= self.instruction_stack_capacity {
            return Err(InstructionError::CallDepth);
        }
        self.instruction_stack.push(index_in_trace);

        Ok(())
    }

    pub fn pop(&mut self) -> Result<(), InstructionError>  {
        if self.instruction_stack.is_empty() {
            return Err(InstructionError::CallDepth);
        }

        if let Some(v) = self.instruction_stack.pop() {
            return Ok(())
        } else {
            return Err(InstructionError::MissingAccount)
        }

    }

    // pub fn update_utxo_authority(&mut self, id: &String,value : &Vec<u8>) -> Result<(), String> {
    //     let mut authority = self.authorities.get_mut(id).ok_or::<String>("no matching authority found".into())?;
    //     *authority = (*value.clone()).to_vec();

    //     Ok(())
    // }

    // pub fn update_utxo_data(&mut self, id: &String,value : &Vec<u8>) -> Result<(), String> {
    //     let mut authority = self.data.get_mut(id).ok_or::<String>("no matching data found".into())?;
    //     *authority = (*value.clone()).to_vec();

    //     Ok(())
    // }
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
    pub(crate) nesting_level : usize,
    pub(crate) instruction_data: Vec<u8>,
    pub(crate) instruction_utxos : Vec<InstructionUtxo>,
    pub(crate) program_id : Pubkey,
}

impl InstructionContext {

    pub fn get_instruction_data(&self) -> &[u8] {
        &self.instruction_data
    }
    pub fn configure(&mut self, instruction_data: &[u8], instruction_utxos: &[InstructionUtxo], program_id : &[u8])  {
        self.instruction_data = instruction_data.to_vec();
        self.instruction_utxos = instruction_utxos.to_vec();
        self.set_program_id(program_id);

    }

    fn set_program_id(&mut self, program_id : &[u8]) {
        self.program_id.as_mut().copy_from_slice(program_id)
    }
    pub fn get_number_of_instruction_utxos(&self) -> usize {
        self.instruction_utxos.len()
    }

    pub fn get_last_program_key(&self) -> &Pubkey {
        &self.program_id
    }

    /// Returns `Some(instruction_autxos_index)` if this is a duplicate
    /// and `None` if it is the first utxo with this key
    pub fn is_instruction_utxo_duplicate(
        &self,
        instruction_account_index: IndexOfUtxo,
    ) -> Result<Option<IndexOfUtxo>, InstructionError> {
        let index_in_callee = self
            .instruction_utxos
            .get(instruction_account_index as usize)
            .ok_or(InstructionError::NotEnoughAccountKeys)?
            .index_in_callee;
        Ok(if index_in_callee == instruction_account_index {
            None
        } else {
            Some(index_in_callee)
        })
    }

     /// Translates the given instruction wide instruction_account_index into a transaction wide index
     pub fn get_index_of_instruction_utxo_in_transaction(
        &self,
        instruction_account_index: IndexOfUtxo,
    ) -> Result<IndexOfUtxo, InstructionError> {
        Ok(self
            .instruction_utxos
            .get(instruction_account_index as usize)
            .ok_or(InstructionError::NotEnoughAccountKeys)?
            .index_in_transaction as IndexOfUtxo)
    }

    /// Gets an instruction utxo of this Instruction
    pub fn try_borrow_instruction_utxo<'a, 'b: 'a>(
        &'a self,
        transaction_context: &'b TransactionContext,
        instruction_account_index: IndexOfUtxo,
    ) -> Result<BorrowedUtxo<'a>, InstructionError> {
        let index_in_transaction =
            self.get_index_of_instruction_utxo_in_transaction(instruction_account_index)?;
        self.try_borrow_utxo(
            transaction_context,
            index_in_transaction,
            instruction_account_index
        )
    }

    fn try_borrow_utxo<'a, 'b: 'a>(
        &'a self,
        transaction_context: &'b TransactionContext,
        index_in_transaction: IndexOfUtxo,
        index_in_instruction: IndexOfUtxo,
    ) -> Result<BorrowedUtxo<'a>, InstructionError> {
        let utxo = transaction_context
            .utxos
            .get(index_in_transaction)
            .ok_or(InstructionError::MissingAccount)?
            .try_borrow_mut()
            .map_err(|_| InstructionError::AccountBorrowFailed)?;

        Ok(BorrowedUtxo {
            transaction_context,
            instruction_context: self,
            index_in_transaction,
            index_in_instruction,
            utxo,
        })
    }
}

/// Shared account borrowed from the TransactionContext and an InstructionContext.
#[derive(Debug)]
pub struct BorrowedUtxo<'a> {
    transaction_context: &'a TransactionContext,
    instruction_context: &'a InstructionContext,
    index_in_transaction: IndexOfUtxo,
    index_in_instruction: IndexOfUtxo,
    utxo: RefMut<'a, UtxoSharedData>,
}

impl<'a> BorrowedUtxo<'a> {
    /// Returns the transaction context
    pub fn transaction_context(&self) -> &TransactionContext {
        self.transaction_context
    }

    /// Returns the index of this account (transaction wide)
    #[inline]
    pub fn get_index_in_transaction(&self) -> IndexOfUtxo {
        self.index_in_transaction
    }

    pub fn data_len(&self) -> usize {
        self.utxo.data.len()
    }

    pub fn get_vout(&self) -> u32 {
        self.utxo.get_vout()
    }

    pub fn get_data(&self) -> &[u8] {
        &self.utxo.data
    }

    pub fn get_txid(&self) -> &[u8] {
        &self.utxo.txid
    }
    
    pub fn get_authority(&self) -> &Pubkey {
        &self.utxo.authority
    }

    pub fn set_authority(&mut self, pubkey : &[u8]) -> Result<(), InstructionError> {
        // Only the owner can assign a new owner
        if !self.is_owned_by_current_utxo() {
            return Err(InstructionError::ModifiedProgramId);
        }

        if self.get_authority().to_bytes() == pubkey {
            return Ok(());
        }

        self.utxo.copy_into_authority_from_slice(pubkey);
        Ok(())
    }

    pub fn can_data_be_resized(&self, new_length: usize) -> Result<(), InstructionError> {
        let old_length = self.get_data().len();
        // Only the owner can change the length of the data
        if new_length != old_length && !self.is_owned_by_current_utxo() {
            return Err(InstructionError::AccountDataSizeChanged);
        }
        // The new length can not exceed the maximum permitted length
        if new_length > MAX_PERMITTED_DATA_LENGTH as usize {
            return Err(InstructionError::InvalidRealloc);
        }

        Ok(())
    }

    pub fn is_owned_by_current_utxo(&self) -> bool {
        self.instruction_context
            .get_last_program_key() == self.get_authority()
            
    }

    pub fn can_data_be_changed(&self) -> Result<(), InstructionError> {
        //  only if we are the owner
        if !self.is_owned_by_current_utxo() {
            return Err(InstructionError::ExternalAccountDataModified);
        }
        Ok(())
    }

    pub fn set_data_from_slice(&mut self, data: &[u8]) -> Result<(), InstructionError> {
        self.can_data_be_resized(data.len())?;
        self.can_data_be_changed()?;
        self.utxo.set_data(data.to_vec());

        Ok(())
    }


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

