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

use std::mem::{self, size_of};

use core_types::{entrypoint::{MAX_PERMITTED_DATA_INCREASE, SUCCESS}, types::{Pubkey, StableInstruction}, utxo_info::UtxoInfo, UtxoId, UtxoIdentity};
use solana_rbpf::{declare_builtin_function, memory_region::{AccessType, MemoryMapping}};

use crate::{config::{translate, translate_slice, translate_slice_mut, translate_type, translate_type_mut}, errors::InstructionError, processor::{BorrowedUtxo, IndexOfUtxo, InstructionUtxo, InvokeContext, SerializedUtxoMetadata, UtxoSharedData}, syscall_error::SyscallError};
type Error = Box<dyn std::error::Error>;

type TranslatedAccounts<'a> = Vec<(IndexOfUtxo, CallerAccount<'a>)>;

/// Host side representation of AccountInfo or SolAccountInfo passed to the CPI syscall.
///
/// At the start of a CPI, this can be different from the data stored in the
/// corresponding BorrowedAccount, and needs to be synched.
struct CallerAccount<'a> {
    authority: &'a mut Pubkey,
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
    ref_to_len_in_vm: VmValue<'a, u64>,
}

enum VmValue<'a, T> {
    Translated(&'a mut T),
}


impl<'a, T> VmValue<'a, T> {
    fn get(&self) -> Result<&T, Error> {
        match self {
            VmValue::Translated(addr) => Ok(*addr),
        }
    }

    fn get_mut(&mut self) -> Result<&mut T, Error> {
        match self {
            VmValue::Translated(addr) => Ok(*addr),
        }
    }
}

impl<'a,'b> CallerAccount<'a> {
    fn from_utxo_info(
        invoke_context: &InvokeContext,
        memory_mapping: &'b MemoryMapping<'a>,
        _vm_addr: u64,
        account_info: &UtxoInfo,
        account_metadata: &SerializedUtxoMetadata,
    ) -> Result<CallerAccount<'a>, Error>  {

        let authority = translate_type_mut::<Pubkey>(
            memory_mapping,
            account_info.authority as *const _ as u64,
            true,
        )?;

        let (serialized_data, vm_data_addr, ref_to_len_in_vm) = {
            // Double translate data out of RefCell
            let data = *translate_type::<&[u8]>(
                memory_mapping,
                account_info.data.as_ptr() as *const _ as u64,
                true,
            )?;

            let ref_to_len_in_vm = {
                let translated = translate(
                    memory_mapping,
                    AccessType::Store,
                    (account_info.data.as_ptr() as *const u64 as u64)
                        .saturating_add(size_of::<u64>() as u64),
                    8,
                )? as *mut u64;
                VmValue::Translated(unsafe { &mut *translated })
            };

            let vm_data_addr = data.as_ptr() as u64;
            
            let serialized_data = 
            translate_slice_mut::<u8>(
                    memory_mapping,
                    vm_data_addr,
                    data.len() as u64,
                    true,
                )?;

                (serialized_data, vm_data_addr, ref_to_len_in_vm)
            };
            
            Ok(CallerAccount {
                authority,
                original_data_len: account_metadata.original_data_len,
                serialized_data,
                vm_data_addr,
                ref_to_len_in_vm,
            })
        }

    // fn get_ref_to_len_in_vm(&self) -> &u64 {

    // }
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


    let mut accounts = S::translate_accounts(&instruction_accounts, account_infos_addr, account_infos_len, memory_mapping, invoke_context)?;

    invoke_context.process_instruction(
        &instruction.data,
        &instruction_accounts,
        &instruction.program_id.as_ref()
    )?;

    // re-bind to please the borrow checker
    let transaction_context = &invoke_context.transaction_context;
    let instruction_context = transaction_context.get_current_instruction_context();


    for (index_in_caller, caller_account) in accounts.iter_mut() {

            let mut callee_account = instruction_context
                .try_borrow_instruction_utxo(transaction_context, *index_in_caller)?;

            update_caller_account(
                invoke_context,
                memory_mapping,
                caller_account,
                &mut callee_account,
            )?;
        
    }

    Ok(SUCCESS)

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
        account_infos_addr: u64,
        account_infos_len: u64,
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
        
        let utxo_metas = translate_slice::<UtxoId>(
            memory_mapping,
            ix.utxos.as_ptr() as u64,
            ix.utxos.len() as u64,
            true,
        )?;

        let mut utxos = Vec::with_capacity(ix.utxos.len());
        utxo_metas.iter().for_each(|utxo_meta| utxos.push(utxo_meta.clone()));

        let data = translate_slice::<u8>(
            memory_mapping,
            ix.data.as_ptr() as u64,
            ix.data.len() as u64,
            true,
        )?
        .to_vec();

        Ok(StableInstruction {
            data: data.into(),
            program_id: ix.program_id,
            utxos: utxos.into(),
        })
    }

    fn translate_accounts<'a, 'b>(
        instruction_accounts: &[InstructionUtxo],
        account_infos_addr: u64,
        account_infos_len: u64,
        memory_mapping: &'b MemoryMapping<'a>,
        invoke_context: &mut InvokeContext,
    ) -> Result<TranslatedAccounts<'a>, Error> {
        
        let (account_infos, account_info_keys) = translate_account_infos(
            account_infos_addr,
            account_infos_len,
            |account_info: &UtxoInfo| (account_info.txid as *const _ as u64, account_info.vout as *const _ as u64),
            memory_mapping,
            invoke_context,
        )?;

        translate_and_update_accounts(
            instruction_accounts,
            &account_info_keys,
            account_infos,
            account_infos_addr,
            invoke_context,
            memory_mapping,
            CallerAccount::from_utxo_info,
        )
    }

}

fn translate_account_infos<'a, T, F>(
    account_infos_addr: u64,
    account_infos_len: u64,
    key_addr: F,
    memory_mapping: &MemoryMapping,
    invoke_context: &mut InvokeContext,
) -> Result<(&'a [T], Vec<UtxoId>), Error>
where
    F: Fn(&T) -> (u64,u64)
{
    let utxo_infos = translate_slice::<T>(
        memory_mapping,
        account_infos_addr,
        account_infos_len,
        true,
    )?;

    let mut account_info_keys = Vec::with_capacity(account_infos_len as usize);

    for account_index in 0..account_infos_len as usize {
        #[allow(clippy::indexing_slicing)]
        let account_info = &utxo_infos[account_index];
        let txid = translate_type::<[u8;32]>(
            memory_mapping,
            key_addr(account_info).0,
            true,
        )?;

        // maybe a problem with this as we store u32 not &u32
        let vout = translate_type::<u32>(
            memory_mapping,
            key_addr(account_info).1,
            true,
        )?;

        // just a way to make this work, could be better
        let utxoid = <UtxoSharedData as UtxoIdentity>::processor(txid, *vout);

        account_info_keys.push(utxoid);
    }
    Ok((utxo_infos, account_info_keys))
}

fn translate_and_update_accounts<'a, 'b, T, F>(
    instruction_accounts: &[InstructionUtxo],
    account_info_keys: &[UtxoId],
    account_infos: &[T],
    account_infos_addr: u64,
    invoke_context: &mut InvokeContext,
    memory_mapping: &'b MemoryMapping<'a>,
    do_translate: F,
) -> Result<TranslatedAccounts<'a>, Error>
where
    F: Fn(
        &InvokeContext,
        &'b MemoryMapping<'a>,
        u64,
        &T,
        &SerializedUtxoMetadata,
    ) ->  Result<CallerAccount<'a>, Error> ,
{
    let transaction_context = &invoke_context.transaction_context;
    let instruction_context = transaction_context.get_current_instruction_context();

    let mut accounts = Vec::with_capacity(instruction_accounts.len());

    let accounts_metadata = &invoke_context
        .get_syscall_context()
        .unwrap()
        .accounts_metadata;

        for (instruction_account_index, instruction_account) in instruction_accounts.iter().enumerate()
        {
            if instruction_account_index as IndexOfUtxo != instruction_account.index_in_callee {
                continue; // Skip duplicate account
            }
    
            let callee_account = instruction_context.try_borrow_instruction_utxo(
                transaction_context,
                instruction_account.index_in_caller,
            )?;

            let account_utxo_id = invoke_context
            .transaction_context
            .get_id_of_utxo_at_index(instruction_account.index_in_transaction)?;

            if let Some(caller_account_index) = account_info_keys.iter().position(|id| id == account_utxo_id) {

                let serialized_metadata = accounts_metadata
                .get(instruction_account.index_in_caller as usize)
                .ok_or_else(|| {
                    Box::new(InstructionError::MissingAccount)
                })?;

                 // build the CallerAccount corresponding to this account.
                if caller_account_index >= account_infos.len() {
                    return Err(Box::new(SyscallError::InvalidLength));
                }

                #[allow(clippy::indexing_slicing)]
                let caller_account =
                    do_translate(
                        invoke_context,
                        memory_mapping,
                        account_infos_addr.saturating_add(
                            caller_account_index.saturating_mul(mem::size_of::<T>()) as u64,
                        ),
                        &account_infos[caller_account_index],
                        serialized_metadata,
                    )?;

                // before initiating CPI, the caller may have modified the
                // account (caller_account). We need to update the corresponding
                // BorrowedAccount (callee_account) so the callee can see the
                // changes.
                update_callee_account(
                    invoke_context,
                    memory_mapping,
                    &caller_account,
                    callee_account,
                )?;

                accounts.push((instruction_account.index_in_caller,caller_account));

            } else {
                return Err(Box::new(SyscallError::InvalidLength));
            }

        }
    Ok(accounts)

}


// Update the given account before executing CPI.
//
// caller_account and callee_account describe the same account. At CPI entry
// caller_account might include changes the caller has made to the account
// before executing CPI.
//
// This method updates callee_account so the CPI callee can see the caller's
// changes.
fn update_callee_account(
    invoke_context: &InvokeContext,
    memory_mapping: &MemoryMapping,
    caller_account: &CallerAccount,
    mut callee_account: BorrowedUtxo<'_>,
)  -> Result<(), Error> {

    // The redundant check helps to avoid the expensive data comparison if we can
    match callee_account
    .can_data_be_resized(caller_account.serialized_data.len())
    .and_then(|_| callee_account.can_data_be_changed())
    {
        Ok(()) => callee_account.set_data_from_slice(caller_account.serialized_data)?,
        Err(err) if callee_account.get_data() != caller_account.serialized_data => {
            return Err(Box::new(err));
        }
        _ => {}
    }

     // Change the owner at the end so that we are allowed to change the lamports and data before
     if callee_account.get_authority() != caller_account.authority {
        callee_account.set_authority(caller_account.authority.as_ref())?;
    }

    Ok(())

}

fn update_caller_account(
    invoke_context: &InvokeContext,
    memory_mapping: &MemoryMapping,
    caller_account: &mut CallerAccount,
    callee_account: &mut BorrowedUtxo<'_>,
) -> Result<(), Error> {

    *caller_account.authority = *callee_account.get_authority();

    let prev_len = *caller_account.ref_to_len_in_vm.get()? as usize;
    let post_len = callee_account.get_data().len();

    if prev_len != post_len {
        let max_increase = MAX_PERMITTED_DATA_INCREASE;

        let data_overflow = post_len
            > caller_account
                .original_data_len
                .saturating_add(max_increase);
        if data_overflow {
            return Err(Box::new(InstructionError::InvalidRealloc));
        }

        // If the account has been shrunk, we're going to zero the unused memory
        // *that was previously used*.
        if post_len < prev_len {
                caller_account
                    .serialized_data
                    .get_mut(post_len..)
                    .ok_or_else(|| Box::new(InstructionError::AccountDataTooSmall))?
                    .fill(0);
            }
        
        caller_account.serialized_data = translate_slice_mut::<u8>(
            memory_mapping,
            caller_account.vm_data_addr,
            post_len as u64,
            false, // Don't care since it is byte aligned
        )?;

        // this is the len field in the AccountInfo::data slice
        *caller_account.ref_to_len_in_vm.get_mut()? = post_len as u64;

        // this is the len field in the serialized parameters
        let serialized_len_ptr = translate_type_mut::<u64>(
            memory_mapping,
            caller_account
                .vm_data_addr
                .saturating_sub(std::mem::size_of::<u64>() as u64),
            true,
        )?;
        *serialized_len_ptr = post_len as u64;
    }


    let to_slice = &mut caller_account.serialized_data;
    let from_slice = callee_account
        .get_data()
        .get(0..post_len)
        .ok_or(SyscallError::InvalidLength)?;
    if to_slice.len() != from_slice.len() {
        return Err(Box::new(InstructionError::AccountDataTooSmall));
    }
    to_slice.copy_from_slice(from_slice);
    Ok(())

}

    

#[test]
fn test() {
    enum VmValue<'a, T> {
        // Once direct mapping is activated, this variant can be removed and the
        // enum can be made a struct.
        Translated(&'a mut T),
    }
    
    impl<'a, T> VmValue<'a, T> {
        fn get(&self) -> Result<&T, Error> {
            match self {
                VmValue::Translated(addr) => Ok(*addr),
            }
        }
    
        fn get_mut(&mut self) -> Result<&mut T, Error> {
            match self {
                VmValue::Translated(addr) => Ok(*addr),
            }
        }
    }

    let b = 64_u64;

    let pointer = &b as *const u64;

    // let a = VmValue::Translated(&mut )
}
