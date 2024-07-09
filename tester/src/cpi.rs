// use std::sync::Arc;

// use solana_rbpf::{memory_region::MemoryMapping, program::SBPFVersion, vm::{Config, EbpfVm, TestContextObject}};

// use crate::config::create_program_runtime_environment_v1;

//  // The Murmur3 hash value (used by RBPF) of the string "entrypoint"
//  const ENTRYPOINT_KEY: u32 = 0x71E3CF81;


//  #[test]
//  fn testing_cpi() {
//     let mut result = create_program_runtime_environment_v1(false).unwrap();

//     let function= result.get_function_registry().lookup_by_key(ENTRYPOINT_KEY).map(|(_name, function)| function).unwrap();

//     let mock_config = Config::default();
//     let empty_memory_mapping =
//             MemoryMapping::new(Vec::new(), &mock_config, &SBPFVersion::V1).unwrap();
    
//             let mut context_object: TestContextObject = TestContextObject::new(15000000000);

//     let mut vm = EbpfVm::new(
//         Arc::new(result),
//         &SBPFVersion::V1,
//         // Removes lifetime tracking
//         &mut context_object,
//         empty_memory_mapping,
//         0,
//     );
    
//     vm.invoke_function(function);

//  }

use core_types::types::{Pubkey, StableInstruction};
use solana_rbpf::{declare_builtin_function, memory_region::MemoryMapping};

use crate::{config::translate_type, processor::{IndexOfUtxo, InstructionUtxo, InvokeContext}};
type Error = Box<dyn std::error::Error>;

type TranslatedAccounts<'a> = Vec<(IndexOfUtxo, Option<CallerAccount<'a>>)>;

/// Host side representation of AccountInfo or SolAccountInfo passed to the CPI syscall.
///
/// At the start of a CPI, this can be different from the data stored in the
/// corresponding BorrowedAccount, and needs to be synched.
struct CallerAccount<'a> {
    lamports: &'a mut u64,
    owner: &'a mut Pubkey,
    // The original data length of the account at the start of the current
    // instruction. We use this to determine wether an account was shrunk or
    // grown before or after CPI, and to derive the vm address of the realloc
    // region.
    original_data_len: usize,
    // This points to the data section for this account, as serialized and
    // mapped inside the vm (see serialize_parameters() in
    // BpfExecutor::execute).
    //
    // This is only set when direct mapping is off (see the relevant comment in
    // CallerAccount::from_account_info).
    serialized_data: &'a mut [u8],
    // Given the corresponding input AccountInfo::data, vm_data_addr points to
    // the pointer field and ref_to_len_in_vm points to the length field.
    vm_data_addr: u64,
    ref_to_len_in_vm: &'a mut u64,
}

declare_builtin_function!(
    /// Cross-program invocation called from Rust
    SyscallInvokeSignedRust,
    fn rust(
        invoke_context: &mut InvokeContext,
        instruction_addr: u64,
        account_infos_addr: u64,
        account_infos_len: u64,
        arg_4: u64,
        arg_5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Error> {
        cpi_common::<Self>(
            invoke_context,
            instruction_addr,
            account_infos_addr,
            account_infos_len,
            memory_mapping,
        )
    }
);


/// Call process instruction, common to both Rust and C
fn cpi_common<S: SyscallInvokeSigned>(
    invoke_context: &mut InvokeContext,
    instruction_addr: u64,
    account_infos_addr: u64,
    account_infos_len: u64,
    memory_mapping: &MemoryMapping,
) -> Result<u64, Error> {
    let instruction = S::translate_instruction(instruction_addr, memory_mapping, invoke_context)?;

    let transaction_context = &invoke_context.transaction_context;
    let instruction_context = transaction_context.get_current_instruction_context();
    let caller_program_id = instruction_context.get_last_program_key();

    let instruction_accounts =
        invoke_context.prepare_instruction(&instruction)?;


    Ok(1)

}

/// Implemented by language specific data structure translators
trait SyscallInvokeSigned {
    fn translate_instruction(
        addr: u64,
        memory_mapping: &MemoryMapping,
        invoke_context: &mut InvokeContext,
    ) -> Result<StableInstruction, Error>;
    fn translate_accounts<'a, 'b>(
        instruction_accounts: &[InstructionUtxo],
        program_indices: &[IndexOfUtxo],
        account_infos_addr: u64,
        account_infos_len: u64,
        is_loader_deprecated: bool,
        memory_mapping: &'b MemoryMapping<'a>,
        invoke_context: &mut InvokeContext,
    ) -> Result<TranslatedAccounts<'a>, Error>;
}

impl SyscallInvokeSigned for SyscallInvokeSignedRust {
    // TODO: Check for max num of ins accounts and tests related to that
    fn translate_instruction(
        addr: u64,
        memory_mapping: &MemoryMapping,
        invoke_context: &mut InvokeContext,
    ) -> Result<StableInstruction, Error> {
        let ix = translate_type::<StableInstruction>(
            memory_mapping,
            addr,
            true,
        )?;
        Err("acb".into())
    }

    fn translate_accounts<'a, 'b>(
        instruction_accounts: &[InstructionUtxo],
        program_indices: &[IndexOfUtxo],
        account_infos_addr: u64,
        account_infos_len: u64,
        is_loader_deprecated: bool,
        memory_mapping: &'b MemoryMapping<'a>,
        invoke_context: &mut InvokeContext,
    ) -> Result<TranslatedAccounts<'a>, Error> {
        todo!()
    }

}