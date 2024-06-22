use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::types::{Instruction, Pubkey, Transaction, UtxoInfo, UtxoMeta};

pub struct MessageProcessor {}

impl MessageProcessor {
    pub fn process_message(
        message : Message,
        log_collector : LogCollector,
        programs : HashMap<String,Vec<u8>>,
        transaction_context : TransactionContext
    ) {
        
    }
}

pub struct InvokeContext {
    transaction_context : TransactionContext,
    log_collector: Option<Rc<RefCell<LogCollector>>>,
    // Not sure if we need both of these
    authority : HashMap<String, Vec<u8>>,
    data: HashMap<String, Vec<u8>>,

}

impl InvokeContext {
    pub fn process_instruction(
        &mut self,
        utxos : Vec<UtxoMeta>,
        program_id : Pubkey,
        instruction_data: Vec<u8>,
    ) -> Result<(), String> {
        self.transaction_context
            .get_next_instruction_context()
            .configure(utxos, program_id, instruction_data);
        self.push()?;
        self.process_executable_chain()
            // MUST pop if and only if `push` succeeded, independent of `result`.
            // Thus, the `.and()` instead of an `.and_then()`.
            .and(self.pop())
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
        self.transaction_context.
        
        Ok(())
    }


}

/// Create the SBF virtual machine
#[macro_export]
macro_rules! create_vm {
    ($vm:ident, $program:expr, $regions:expr, $accounts_metadata:expr, $invoke_context:expr $(,)?) => {
        let invoke_context = &*$invoke_context;
        let stack_size = $program.get_config().stack_size();
        let heap_size = invoke_context.get_compute_budget().heap_size;
        let mut allocations = None;
        let $vm = heap_cost_result.and_then(|_| {
            let mut stack = solana_rbpf::aligned_memory::AlignedMemory::<
                { solana_rbpf::ebpf::HOST_ALIGN },
            >::zero_filled(stack_size);
            let mut heap = solana_rbpf::aligned_memory::AlignedMemory::<
                { solana_rbpf::ebpf::HOST_ALIGN },
            >::zero_filled(usize::try_from(heap_size).unwrap());
            let vm = $crate::create_vm(
                $program,
                $regions,
                $accounts_metadata,
                $invoke_context,
                &mut stack,
                &mut heap,
            );
            allocations = Some((stack, heap));
            vm
        });
    };
}

#[derive(Debug, Clone)]
pub struct Message {
    pub signers: Vec<Pubkey>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub struct TransactionContext {
    instructions: Vec<Instruction>,
    instruction_stack_capacity: usize,
    instruction_trace_capacity: usize,
    instruction_stack: Vec<usize>,
    instruction_trace: Vec<InstructionContext>,
    authorities : HashMap<String,Vec<u8>>,
    data : HashMap<String,Vec<u8>>
}

impl TransactionContext {
    pub fn new(
        instructions : Vec<Instruction>,
        instruction_stack_capacity: usize,
        instruction_trace_capacity: usize,
        authorities : HashMap<String,Vec<u8>>,
        data : HashMap<String,Vec<u8>>
    ) -> Self {
            Self {
                instructions,
                instruction_stack_capacity,
                instruction_trace_capacity,
                instruction_stack: Vec::with_capacity(instruction_stack_capacity),
                instruction_trace: vec![InstructionContext::default()],
                authorities,
                data
            }
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
        *authority = *value;

        Ok(())
    }

    pub fn update_utxo_data(&mut self, id: &String,value : &Vec<u8>) -> Result<(), String> {
        let mut authority = self.data.get_mut(id).ok_or::<String>("no matching data found".into())?;
        *authority = *value;

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
    utxos : Vec<UtxoMeta>,
    program_id : Pubkey,
    instruction_data: Vec<u8>,
}

impl InstructionContext {
    pub fn configure(&mut self, utxos: Vec<UtxoMeta>,program_id : Pubkey, instruction_data: Vec<u8>)  {
        self.utxos = utxos;
        self.program_id = program_id;
        self.instruction_data  = instruction_data;
    }
}

// #[derive(Debug, Clone)]
// pub struct BorrowedUTXO {
//     id : String,
//     authority : Pubkey,
//     data: Vec<u8>,
// }

pub fn process_instruction() {
    // write all accounts involved into the locations

    ///Format
    /// Instruction + Authorities + Data
    serealise() 
}

fn serealise(transaction_context : TransactionContext) -> Vec<u8>{
    let current_context = transaction_context.get_current_instruction_context();
    let (utxo_authorities, utxo_data) = (transaction_context.authorities,transaction_context.data);

    let instruction = Instruction{
        program_id: current_context.program_id,
        utxos: current_context.utxos,
        data: current_context.instruction_data
    };

    let mut serialised_data = borsh::to_vec(&(instruction,utxo_authorities,utxo_data)).unwrap();
    let mut data_len = serialised_data.len() as u32;
    let mut data_len = data_len.to_le_bytes().to_vec();
    
    data_len.append(&mut serialised_data);
    data_len
}

fn deserialise(mut transaction_context : TransactionContext, mem : Vec<u8>) -> Transaction {
    let instruction_context = transaction_context.get_current_instruction_context();
    let current_program_id = instruction_context.program_id;

    let length = [mem[0],mem[1],mem[2],mem[3]];
    let length_of_output = u32::from_le_bytes(length);

    let (output_utxos, transaction)  = borsh::from_slice::<(Vec<UtxoInfo>,Transaction)>(&mem[4..(length_of_output + 4) as usize]).expect("can't deser");

    // for (&id, authority) in transaction_context.authorities.iter_mut() {
    //     for output_utxo in output_utxos {
    //         if output_utxo.id() == id && current_program_id.0 == *authority {

    //         }
    //     }
    // }

    // update authorities after checking if program could update the authorities
    for output_utxo in output_utxos {
        let output_utox_id = output_utxo.id();
        let utxo_authority_in_context = transaction_context.authorities.get(&output_utox_id).expect("must have a key in authority");
        if *utxo_authority_in_context == current_program_id.0 {

            // checking here can save us updating hashmap cost
            if output_utxo.authority.into_inner().0 != *transaction_context.authorities.get(&output_utox_id).unwrap() {
                transaction_context.update_utxo_authority(&output_utox_id,&output_utxo.authority.into_inner().0);
            }
            transaction_context.update_utxo_data(&output_utox_id,&output_utxo.data.into_inner());
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