use std::{alloc::Layout, mem::{align_of, size_of}, slice::{self, from_raw_parts_mut}, str::from_utf8};

use solana_rbpf::{declare_builtin_function, error::EbpfError, memory_region::{AccessType, MemoryMapping, MemoryRegion}, program::{BuiltinFunction, BuiltinProgram, FunctionRegistry}, vm::{Config, TestContextObject}};

// use crate::processor::InvokeContext;

// use crate::{cpi::CpiContext, processor::InvokeContext};
use crate::{ processor::InvokeContext};

type Error = Box<dyn std::error::Error>;

pub fn create_program_runtime_environment_v1<'a>(
    reject_deployment_of_broken_elfs: bool,
) -> Result<BuiltinProgram<InvokeContext<'a>>, Error> {

    let config = Config {
        max_call_depth: 20,
        stack_frame_size: 4096,
        enable_address_translation: true,
        enable_stack_frame_gaps: false,
        instruction_meter_checkpoint_distance: 10000,
        enable_instruction_meter: true,
        enable_instruction_tracing: true,
        enable_symbol_and_section_labels: true,
        reject_broken_elfs: reject_deployment_of_broken_elfs,
        noop_instruction_rate: 256,
        sanitize_user_provided_values: true,
        external_internal_function_hash_collision: true,
        reject_callx_r10: true,
        enable_sbpf_v1: true,
        enable_sbpf_v2: false,
        optimize_rodata: false,
        new_elf_parser: false,
        aligned_memory_mapping: true,
        // Warning, do not use `Config::default()` so that configuration here is explicit.
    };
    let mut result = FunctionRegistry::<BuiltinFunction<InvokeContext>>::default();

    // Abort
    result.register_function_hashed(*b"abort", SyscallAbort::vm)?;

    // Panic
    result.register_function_hashed(*b"sol_panic_", SyscallPanic::vm)?;

    // Logging
    result.register_function_hashed(*b"sol_log_", SyscallLog::vm)?;
    // result.register_function_hashed(*b"sol_log_64_", SyscallLogU64::vm)?;
    result.register_function_hashed(*b"sol_log_compute_units_", SyscallLogBpfComputeUnits::vm)?;
    // result.register_function_hashed(*b"sol_log_pubkey", SyscallLogPubkey::vm)?;

    // Program defined addresses (PDA)
    // result.register_function_hashed(
    //     *b"sol_create_program_address",
    //     SyscallCreateProgramAddress::vm,
    // )?;

    // result.register_function_hashed(
    //     *b"sol_try_find_program_address",
    //     SyscallTryFindProgramAddress::vm,
    // )?;

    // Sha256
    // result.register_function_hashed(*b"sol_sha256", SyscallHash::vm::<Sha256Hasher>)?;

    // Keccak256
    // result.register_function_hashed(*b"sol_keccak256", SyscallHash::vm::<Keccak256Hasher>)?;

    // Secp256k1 Recover
    // result.register_function_hashed(*b"sol_secp256k1_recover", SyscallSecp256k1Recover::vm)?;

    // Blake3
    // register_feature_gated_function!(
    //     result,
    //     blake3_syscall_enabled,
    //     *b"sol_blake3",
    //     SyscallHash::vm::<Blake3Hasher>,
    // )?;

    // Elliptic Curve Operations
    // register_feature_gated_function!(
    //     result,
    //     curve25519_syscall_enabled,
    //     *b"sol_curve_validate_point",
    //     SyscallCurvePointValidation::vm,
    // )?;
    // register_feature_gated_function!(
    //     result,
    //     curve25519_syscall_enabled,
    //     *b"sol_curve_group_op",
    //     SyscallCurveGroupOps::vm,
    // )?;
    // register_feature_gated_function!(
    //     result,
    //     curve25519_syscall_enabled,
    //     *b"sol_curve_multiscalar_mul",
    //     SyscallCurveMultiscalarMultiplication::vm,
    // )?;

    // // Sysvars
    // result.register_function_hashed(*b"sol_get_clock_sysvar", SyscallGetClockSysvar::vm)?;

    // result.register_function_hashed(
    //     *b"sol_get_epoch_schedule_sysvar",
    //     SyscallGetEpochScheduleSysvar::vm,
    // )?;

    // register_feature_gated_function!(
    //     result,
    //     !disable_fees_sysvar,
    //     *b"sol_get_fees_sysvar",
    //     SyscallGetFeesSysvar::vm,
    // )?;

    // result.register_function_hashed(*b"sol_get_rent_sysvar", SyscallGetRentSysvar::vm)?;

    // register_feature_gated_function!(
    //     result,
    //     last_restart_slot_syscall_enabled,
    //     *b"sol_get_last_restart_slot",
    //     SyscallGetLastRestartSlotSysvar::vm,
    // )?;

    // register_feature_gated_function!(
    //     result,
    //     epoch_rewards_syscall_enabled,
    //     *b"sol_get_epoch_rewards_sysvar",
    //     SyscallGetEpochRewardsSysvar::vm,
    // )?;

    // Memory ops
    result.register_function_hashed(*b"sol_memcpy_", SyscallMemcpy::vm)?;
    result.register_function_hashed(*b"sol_memmove_", SyscallMemmove::vm)?;
    result.register_function_hashed(*b"sol_memcmp_", SyscallMemcmp::vm)?;
    result.register_function_hashed(*b"sol_memset_", SyscallMemset::vm)?;

    // Processed sibling instructions
    // result.register_function_hashed(
    //     *b"sol_get_processed_sibling_instruction",
    //     SyscallGetProcessedSiblingInstruction::vm,
    // )?;

    // Stack height
    // result.register_function_hashed(*b"sol_get_stack_height", SyscallGetStackHeight::vm)?;

    // Return data
    result.register_function_hashed(*b"sol_set_return_data", SyscallSetReturnData::vm)?;
    // result.register_function_hashed(*b"sol_get_return_data", SyscallGetReturnData::vm)?;

    // Cross-program invocation
    // result.register_function_hashed(*b"sol_invoke_signed_c", SyscallInvokeSignedC::vm)?;
    // result.register_function_hashed(*b"sol_invoke_signed_rust", SyscallInvokeSignedRust::vm)?;

    // Memory allocator
    // register_feature_gated_function!(
    //     result,
    //     true,
    //     *b"sol_alloc_free_",
    //     SyscallAllocFree::vm,
    // )?;

    result.register_function_hashed( *b"sol_alloc_free_",SyscallAllocFree::vm)?;

    // // Alt_bn128
    // register_feature_gated_function!(
    //     result,
    //     enable_alt_bn128_syscall,
    //     *b"sol_alt_bn128_group_op",
    //     SyscallAltBn128::vm,
    // )?;

    // // Big_mod_exp
    // register_feature_gated_function!(
    //     result,
    //     enable_big_mod_exp_syscall,
    //     *b"sol_big_mod_exp",
    //     SyscallBigModExp::vm,
    // )?;

    // // Poseidon
    // register_feature_gated_function!(
    //     result,
    //     enable_poseidon_syscall,
    //     *b"sol_poseidon",
    //     SyscallPoseidon::vm,
    // )?;

    // // Accessing remaining compute units
    // register_feature_gated_function!(
    //     result,
    //     remaining_compute_units_syscall_enabled,
    //     *b"sol_remaining_compute_units",
    //     SyscallRemainingComputeUnits::vm
    // )?;

    // // Alt_bn128_compression
    // register_feature_gated_function!(
    //     result,
    //     enable_alt_bn128_compression_syscall,
    //     *b"sol_alt_bn128_compression",
    //     SyscallAltBn128Compression::vm,
    // )?;

    // Log data
    // result.register_function_hashed(*b"sol_log_data", SyscallLogData::vm)?;

    Ok(BuiltinProgram::new_loader(config, result))
}

pub struct SyscallInvokeSignedRust {}

// pub trait SyscallInvokeSigned {
//     fn translate_instruction(
//         addr: u64,
//         memory_mapping: &MemoryMapping,
//         invoke_context: &mut InvokeContext,
//     ) -> Result<CpiContext, Error>;
// }

// impl SyscallInvokeSigned for SyscallInvokeSignedRust {
//     fn translate_instruction(
//         addr: u64,
//         memory_mapping: &MemoryMapping,
//         invoke_context: &mut InvokeContext,
//     ) -> Result<CpiContext, Error> {
//         let context = translate_type::<CpiContext>(
//             memory_mapping,
//             addr,
//             true,
//         )?;
//         Ok(*context)
//     }
// }
fn address_is_aligned<T>(address: u64) -> bool {
    (address as *mut T as usize)
        .checked_rem(align_of::<T>())
        .map(|rem| rem == 0)
        .expect("T to be non-zero aligned")
}

fn translate(
    memory_mapping: &MemoryMapping,
    access_type: AccessType,
    vm_addr: u64,
    len: u64,
) -> Result<u64, Error> {
    memory_mapping
        .map(access_type, vm_addr, len)
        .map_err(|err| err.into())
        .into()
}

fn translate_type_inner<'a, T>(
    memory_mapping: &MemoryMapping,
    access_type: AccessType,
    vm_addr: u64,
    check_aligned: bool,
) -> Result<&'a mut T, Error> {
    let host_addr = translate(memory_mapping, access_type, vm_addr, size_of::<T>() as u64)?;
    if !check_aligned {
        Ok(unsafe { std::mem::transmute::<u64, &mut T>(host_addr) })
    } else if !address_is_aligned::<T>(host_addr) {
        Err("UnalignedPointer".into())
    } else {
        Ok(unsafe { &mut *(host_addr as *mut T) })
    }
}
fn translate_type_mut<'a, T>(
    memory_mapping: &MemoryMapping,
    vm_addr: u64,
    check_aligned: bool,
) -> Result<&'a mut T, Error> {
    translate_type_inner::<T>(memory_mapping, AccessType::Store, vm_addr, check_aligned)
}
fn translate_type<'a, T>(
    memory_mapping: &MemoryMapping,
    vm_addr: u64,
    check_aligned: bool,
) -> Result<&'a T, Error> {
    translate_type_inner::<T>(memory_mapping, AccessType::Load, vm_addr, check_aligned)
        .map(|value| &*value)
}

fn translate_slice_inner<'a, T>(
    memory_mapping: &MemoryMapping,
    access_type: AccessType,
    vm_addr: u64,
    len: u64,
    check_aligned: bool,
) -> Result<&'a mut [T], Error> {
    if len == 0 {
        return Ok(&mut []);
    }

    let total_size = len.saturating_mul(size_of::<T>() as u64);
    if isize::try_from(total_size).is_err() {
        return Err("InvalidLength".into());
    }

    let host_addr = translate(memory_mapping, access_type, vm_addr, total_size)?;

    if check_aligned && !address_is_aligned::<T>(host_addr) {
        return Err("UnalignedPointer".into());
    }
    Ok(unsafe { from_raw_parts_mut(host_addr as *mut T, len as usize) })
}
fn translate_slice_mut<'a, T>(
    memory_mapping: &MemoryMapping,
    vm_addr: u64,
    len: u64,
    check_aligned: bool,
) -> Result<&'a mut [T], Error> {
    translate_slice_inner::<T>(
        memory_mapping,
        AccessType::Store,
        vm_addr,
        len,
        check_aligned,
    )
}
fn translate_slice<'a, T>(
    memory_mapping: &MemoryMapping,
    vm_addr: u64,
    len: u64,
    check_aligned: bool,
) -> Result<&'a [T], Error> {
    translate_slice_inner::<T>(
        memory_mapping,
        AccessType::Load,
        vm_addr,
        len,
        check_aligned,
    )
    .map(|value| &*value)
}

/// Take a virtual pointer to a string (points to SBF VM memory space), translate it
/// pass it to a user-defined work function
fn translate_string_and_do(
    memory_mapping: &MemoryMapping,
    addr: u64,
    len: u64,
    check_aligned: bool,
    work: &mut dyn FnMut(&str) -> Result<u64, Error>,
) -> Result<u64, Error> {
    let buf = translate_slice::<u8>(memory_mapping, addr, len, check_aligned)?;
    match from_utf8(buf) {
        Ok(message) => work(message),
        Err(err) => Err(format!("InvalidString {:?} {:?}",err,buf.to_vec()).into()),
    }
}

declare_builtin_function!(
    /// Abort syscall functions, called when the SBF program calls `abort()`
    /// LLVM will insert calls to `abort()` if it detects an untenable situation,
    /// `abort()` is not intended to be called explicitly by the program.
    /// Causes the SBF program to be halted immediately
    SyscallAbort,
    fn rust(
        _invoke_context: &mut InvokeContext,
        _arg1: u64,
        _arg2: u64,
        _arg3: u64,
        _arg4: u64,
        _arg5: u64,
        _memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Error> {
        Err("memoryerror".into())
    }
);
declare_builtin_function!(
    /// Log current compute consumption
    SyscallLogBpfComputeUnits,
    fn rust(
        invoke_context: &mut InvokeContext,
        _arg1: u64,
        _arg2: u64,
        _arg3: u64,
        _arg4: u64,
        _arg5: u64,
        _memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Error> {
        // let cost = invoke_context.get_compute_budget().syscall_base_cost;
        // consume_compute_meter(invoke_context, cost)?;

        // ic_logger_msg!(
        //     invoke_context.get_log_collector(),
        //     "Program consumption: {} units remaining",
        //     invoke_context.get_remaining(),
        // );
        Ok(0)
    }
);
declare_builtin_function!(
    /// Set return data
    SyscallSetReturnData,
    fn rust(
        invoke_context: &mut InvokeContext,
        addr: u64,
        len: u64,
        _arg3: u64,
        _arg4: u64,
        _arg5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Error> {

        // if len > MAX_RETURN_DATA as u64 {
        //     return Err(SyscallError::ReturnDataTooLarge(len, MAX_RETURN_DATA as u64).into());
        // }

        // let return_data = if len == 0 {
        //     Vec::new()
        // } else {
        //     translate_slice::<u8>(
        //         memory_mapping,
        //         addr,
        //         len,
        //         // invoke_context.get_check_aligned(),
        //         true
        //     )?
        //     .to_vec()
        // };
        // let transaction_context = &mut invoke_context.transaction_context;
        // let program_id = *transaction_context
        //     .get_current_instruction_context()
        //     .and_then(|instruction_context| {
        //         instruction_context.get_last_program_key(transaction_context)
        //     })?;

        // transaction_context.set_return_data(program_id, return_data)?;

        Ok(0)
    }
);
declare_builtin_function!(
    /// Panic syscall function, called when the SBF program calls 'sol_panic_()`
    /// Causes the SBF program to be halted immediately
    SyscallPanic,
    fn rust(
        invoke_context: &mut InvokeContext,
        file: u64,
        len: u64,
        line: u64,
        column: u64,
        _arg5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Error> {

        translate_string_and_do(
            memory_mapping,
            file,
            len,
            true,
            &mut |string: &str| Err(
                format!("SyscallError::Panic {:?} {:?} {:?}", string.to_string(), line, column).into()),
        )
    }
);

// declare_builtin_function!(
//     /// Cross-program invocation called from Rust
//     SyscallInvokeSignedRust,
//     fn rust(
//         invoke_context: &mut InvokeContext,
//         instruction_addr: u64,
//         account_infos_addr: u64,
//         // account_infos_len: u64,
//         // signers_seeds_addr: u64,
//         // signers_seeds_len: u64,
//         _arg5: u64,
//         memory_mapping: &mut MemoryMapping,
//     ) -> Result<u64, Error> {
//         cpi_common::<Self>(
//             invoke_context,
//             memory_mapping,
//         )
//     }
// );


declare_builtin_function!(
    /// Log a user's info message
    SyscallLog,
    fn rust(
        invoke_context: &mut InvokeContext,
        addr: u64,
        len: u64,
        _arg3: u64,
        _arg4: u64,
        _arg5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Error> {
        let buf = translate_slice::<u8>(memory_mapping, addr, len, true)?;
        match from_utf8(buf) {
            Ok(message) => {
                log::debug!("Output ");
                        Ok(0)
            }
            Err(err) => Err(format!("InvalidString {:?} {:?}",err,buf.to_vec()).into()),
        }
    }
);


declare_builtin_function!(
    /// Dynamic memory allocation syscall called when the SBF program calls
    /// `sol_alloc_free_()`.  The allocator is expected to allocate/free
    /// from/to a given chunk of memory and enforce size restrictions.  The
    /// memory chunk is given to the allocator during allocator creation and
    /// information about that memory (start address and size) is passed
    /// to the VM to use for enforcement.
    SyscallAllocFree,
    fn rust(
        invoke_context: &mut InvokeContext,
        size: u64,
        free_addr: u64,
        _arg3: u64,
        _arg4: u64,
        _arg5: u64,
        _memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Error> {
        // TODO : HARDCODED to 8, change it 
        let Ok(layout) = Layout::from_size_align(size as usize, 8) else {
            return Ok(0);
        };
        let allocator = &mut invoke_context.get_syscall_context_mut()?.allocator;
        if free_addr == 0 {
            match allocator.alloc(layout) {
                Ok(addr) => Ok(addr),
                Err(_) => Ok(0),
            }
        } else {
            // Unimplemented
            Ok(0)
        }
    }
);




// fn mem_op_consume(invoke_context: &mut InvokeContext, n: u64) -> Result<(), Error> {
//     let compute_budget = invoke_context.get_compute_budget();
//     let cost = compute_budget.mem_op_base_cost.max(
//         n.checked_div(compute_budget.cpi_bytes_per_unit)
//             .unwrap_or(u64::MAX),
//     );
//     // consume_compute_meter(invoke_context, cost)
// }

declare_builtin_function!(
    /// memcpy
    SyscallMemcpy,
    fn rust(
        invoke_context: &mut InvokeContext,
        dst_addr: u64,
        src_addr: u64,
        n: u64,
        _arg4: u64,
        _arg5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Error> {
        // mem_op_consume(invoke_context, n)?;

        if !is_nonoverlapping(src_addr, n, dst_addr, n) {
            return Err(format!("SyscallError::CopyOverlapping").into());
        }

        // host addresses can overlap so we always invoke memmove
        memmove(invoke_context, dst_addr, src_addr, n, memory_mapping)
    }
);

declare_builtin_function!(
    /// memmove
    SyscallMemmove,
    fn rust(
        invoke_context: &mut InvokeContext,
        dst_addr: u64,
        src_addr: u64,
        n: u64,
        _arg4: u64,
        _arg5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Error> {
        // mem_op_consume(invoke_context, n)?;

        memmove(invoke_context, dst_addr, src_addr, n, memory_mapping)
    }
);

declare_builtin_function!(
    /// memcmp
    SyscallMemcmp,
    fn rust(
        invoke_context: &mut InvokeContext,
        s1_addr: u64,
        s2_addr: u64,
        n: u64,
        cmp_result_addr: u64,
        _arg5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Error> {
        // mem_op_consume(invoke_context, n)?;

            let cmp_result = translate_type_mut::<i32>(
                memory_mapping,
                cmp_result_addr,
                // invoke_context.get_check_aligned(),
                true,
            )?;
            *cmp_result = memcmp_non_contiguous(s1_addr, s2_addr, n, memory_mapping)?;
        // } else {
        //     let s1 = translate_slice::<u8>(
        //         memory_mapping,
        //         s1_addr,
        //         n,
        //         invoke_context.get_check_aligned(),
        //     )?;
        //     let s2 = translate_slice::<u8>(
        //         memory_mapping,
        //         s2_addr,
        //         n,
        //         invoke_context.get_check_aligned(),
        //     )?;
        //     let cmp_result = translate_type_mut::<i32>(
        //         memory_mapping,
        //         cmp_result_addr,
        //         invoke_context.get_check_aligned(),
        //     )?;

        //     debug_assert_eq!(s1.len(), n as usize);
        //     debug_assert_eq!(s2.len(), n as usize);
        //     // Safety:
        //     // memcmp is marked unsafe since it assumes that the inputs are at least
        //     // `n` bytes long. `s1` and `s2` are guaranteed to be exactly `n` bytes
        //     // long because `translate_slice` would have failed otherwise.
        //     *cmp_result = unsafe { memcmp(s1, s2, n as usize) };
        // }

        Ok(0)
    }
);

declare_builtin_function!(
    /// memset
    SyscallMemset,
    fn rust(
        invoke_context: &mut InvokeContext,
        dst_addr: u64,
        c: u64,
        n: u64,
        _arg4: u64,
        _arg5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Error> {
        // mem_op_consume(invoke_context, n)?;

        // if invoke_context
        //     .feature_set
        //     .is_active(&feature_set::bpf_account_data_direct_mapping::id())
        // {
            memset_non_contiguous(dst_addr, c as u8, n, memory_mapping)
        // } else {
        //     let s = translate_slice_mut::<u8>(
        //         memory_mapping,
        //         dst_addr,
        //         n,
        //         invoke_context.get_check_aligned(),
        //     )?;
        //     s.fill(c as u8);
        //     Ok(0)
        // }
    }
);

#[macro_export]
macro_rules! register_feature_gated_function {
    ($result:expr, $is_feature_active:expr, $name:expr, $call:expr $(,)?) => {
        if $is_feature_active {
            $result.register_function_hashed($name, $call)
        } else {
            Ok(0)
        }
    };
}

fn memmove(
    invoke_context: &mut InvokeContext,
    dst_addr: u64,
    src_addr: u64,
    n: u64,
    memory_mapping: &MemoryMapping,
) -> Result<u64, Error> {
    // if invoke_context
    //     .feature_set
    //     .is_active(&feature_set::bpf_account_data_direct_mapping::id())
    // {
        memmove_non_contiguous(dst_addr, src_addr, n, memory_mapping)
    // } else {
    //     let dst_ptr = translate_slice_mut::<u8>(
    //         memory_mapping,
    //         dst_addr,
    //         n,
    //         invoke_context.get_check_aligned(),
    //     )?
    //     .as_mut_ptr();
    //     let src_ptr = translate_slice::<u8>(
    //         memory_mapping,
    //         src_addr,
    //         n,
    //         invoke_context.get_check_aligned(),
    //     )?
    //     .as_ptr();

    //     unsafe { std::ptr::copy(src_ptr, dst_ptr, n as usize) };
    //     Ok(0)
    // }
}

fn memmove_non_contiguous(
    dst_addr: u64,
    src_addr: u64,
    n: u64,
    memory_mapping: &MemoryMapping,
) -> Result<u64, Error> {
    let reverse = dst_addr.wrapping_sub(src_addr) < n;
    iter_memory_pair_chunks(
        AccessType::Load,
        src_addr,
        AccessType::Store,
        dst_addr,
        n,
        memory_mapping,
        reverse,
        |src_host_addr, dst_host_addr, chunk_len| {
            unsafe { std::ptr::copy(src_host_addr, dst_host_addr as *mut u8, chunk_len) };
            Ok(0)
        },
    )
}

// Marked unsafe since it assumes that the slices are at least `n` bytes long.
unsafe fn memcmp(s1: &[u8], s2: &[u8], n: usize) -> i32 {
    for i in 0..n {
        let a = *s1.get_unchecked(i);
        let b = *s2.get_unchecked(i);
        if a != b {
            return (a as i32).saturating_sub(b as i32);
        };
    }

    0
}

fn memcmp_non_contiguous(
    src_addr: u64,
    dst_addr: u64,
    n: u64,
    memory_mapping: &MemoryMapping,
) -> Result<i32, Error> {
    let memcmp_chunk = |s1_addr, s2_addr, chunk_len| {
        let res = unsafe {
            let s1 = slice::from_raw_parts(s1_addr, chunk_len);
            let s2 = slice::from_raw_parts(s2_addr, chunk_len);
            // Safety:
            // memcmp is marked unsafe since it assumes that s1 and s2 are exactly chunk_len
            // long. The whole point of iter_memory_pair_chunks is to find same length chunks
            // across two memory regions.
            memcmp(s1, s2, chunk_len)
        };
        if res != 0 {
            return Err(MemcmpError::Diff(res).into());
        }
        Ok(0)
    };
    match iter_memory_pair_chunks(
        AccessType::Load,
        src_addr,
        AccessType::Load,
        dst_addr,
        n,
        memory_mapping,
        false,
        memcmp_chunk,
    ) {
        Ok(res) => Ok(res),
        Err(error) => match error.downcast_ref() {
            Some(MemcmpError::Diff(diff)) => Ok(*diff),
            _ => Err(error),
        },
    }
}

#[derive(Debug)]
enum MemcmpError {
    Diff(i32),
}

impl std::fmt::Display for MemcmpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemcmpError::Diff(diff) => write!(f, "memcmp diff: {diff}"),
        }
    }
}

impl std::error::Error for MemcmpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MemcmpError::Diff(_) => None,
        }
    }
}

fn memset_non_contiguous(
    dst_addr: u64,
    c: u8,
    n: u64,
    memory_mapping: &MemoryMapping,
) -> Result<u64, Error> {
    let dst_chunk_iter = MemoryChunkIterator::new(memory_mapping, AccessType::Store, dst_addr, n)?;
    for item in dst_chunk_iter {
        let (dst_region, dst_vm_addr, dst_len) = item?;
        let dst_host_addr = Result::from(dst_region.vm_to_host(dst_vm_addr, dst_len as u64))?;
        unsafe { slice::from_raw_parts_mut(dst_host_addr as *mut u8, dst_len).fill(c) }
    }

    Ok(0)
}

fn iter_memory_pair_chunks<T, F>(
    src_access: AccessType,
    src_addr: u64,
    dst_access: AccessType,
    dst_addr: u64,
    n_bytes: u64,
    memory_mapping: &MemoryMapping,
    reverse: bool,
    mut fun: F,
) -> Result<T, Error>
where
    T: Default,
    F: FnMut(*const u8, *const u8, usize) -> Result<T, Error>,
{
    let mut src_chunk_iter =
        MemoryChunkIterator::new(memory_mapping, src_access, src_addr, n_bytes)
            .map_err(EbpfError::from)?;
    let mut dst_chunk_iter =
        MemoryChunkIterator::new(memory_mapping, dst_access, dst_addr, n_bytes)
            .map_err(EbpfError::from)?;

    let mut src_chunk = None;
    let mut dst_chunk = None;

    macro_rules! memory_chunk {
        ($chunk_iter:ident, $chunk:ident) => {
            if let Some($chunk) = &mut $chunk {
                // Keep processing the current chunk
                $chunk
            } else {
                // This is either the first call or we've processed all the bytes in the current
                // chunk. Move to the next one.
                let chunk = match if reverse {
                    $chunk_iter.next_back()
                } else {
                    $chunk_iter.next()
                } {
                    Some(item) => item?,
                    None => break,
                };
                $chunk.insert(chunk)
            }
        };
    }

    loop {
        let (src_region, src_chunk_addr, src_remaining) = memory_chunk!(src_chunk_iter, src_chunk);
        let (dst_region, dst_chunk_addr, dst_remaining) = memory_chunk!(dst_chunk_iter, dst_chunk);

        // We always process same-length pairs
        let chunk_len = *src_remaining.min(dst_remaining);

        let (src_host_addr, dst_host_addr) = {
            let (src_addr, dst_addr) = if reverse {
                // When scanning backwards not only we want to scan regions from the end,
                // we want to process the memory within regions backwards as well.
                (
                    src_chunk_addr
                        .saturating_add(*src_remaining as u64)
                        .saturating_sub(chunk_len as u64),
                    dst_chunk_addr
                        .saturating_add(*dst_remaining as u64)
                        .saturating_sub(chunk_len as u64),
                )
            } else {
                (*src_chunk_addr, *dst_chunk_addr)
            };

            (
                Result::from(src_region.vm_to_host(src_addr, chunk_len as u64))?,
                Result::from(dst_region.vm_to_host(dst_addr, chunk_len as u64))?,
            )
        };

        fun(
            src_host_addr as *const u8,
            dst_host_addr as *const u8,
            chunk_len,
        )?;

        // Update how many bytes we have left to scan in each chunk
        *src_remaining = src_remaining.saturating_sub(chunk_len);
        *dst_remaining = dst_remaining.saturating_sub(chunk_len);

        if !reverse {
            // We've scanned `chunk_len` bytes so we move the vm address forward. In reverse
            // mode we don't do this since we make progress by decreasing src_len and
            // dst_len.
            *src_chunk_addr = src_chunk_addr.saturating_add(chunk_len as u64);
            *dst_chunk_addr = dst_chunk_addr.saturating_add(chunk_len as u64);
        }

        if *src_remaining == 0 {
            src_chunk = None;
        }

        if *dst_remaining == 0 {
            dst_chunk = None;
        }
    }

    Ok(T::default())
}

struct MemoryChunkIterator<'a> {
    memory_mapping: &'a MemoryMapping<'a>,
    access_type: AccessType,
    initial_vm_addr: u64,
    vm_addr_start: u64,
    // exclusive end index (start + len, so one past the last valid address)
    vm_addr_end: u64,
    len: u64,
}

impl<'a> MemoryChunkIterator<'a> {
    fn new(
        memory_mapping: &'a MemoryMapping,
        access_type: AccessType,
        vm_addr: u64,
        len: u64,
    ) -> Result<MemoryChunkIterator<'a>, EbpfError> {
        let vm_addr_end = vm_addr.checked_add(len).ok_or(EbpfError::AccessViolation(
            access_type,
            vm_addr,
            len,
            "unknown",
        ))?;
        Ok(MemoryChunkIterator {
            memory_mapping,
            access_type,
            initial_vm_addr: vm_addr,
            len,
            vm_addr_start: vm_addr,
            vm_addr_end,
        })
    }

    fn region(&mut self, vm_addr: u64) -> Result<&'a MemoryRegion, Error> {
        match self.memory_mapping.region(self.access_type, vm_addr) {
            Ok(region) => Ok(region),
            Err(error) => match error {
                EbpfError::AccessViolation(access_type, _vm_addr, _len, name) => Err(Box::new(
                    EbpfError::AccessViolation(access_type, self.initial_vm_addr, self.len, name),
                )),
                EbpfError::StackAccessViolation(access_type, _vm_addr, _len, frame) => {
                    Err(Box::new(EbpfError::StackAccessViolation(
                        access_type,
                        self.initial_vm_addr,
                        self.len,
                        frame,
                    )))
                }
                _ => Err(error.into()),
            },
        }
    }
}

impl<'a> Iterator for MemoryChunkIterator<'a> {
    type Item = Result<(&'a MemoryRegion, u64, usize), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.vm_addr_start == self.vm_addr_end {
            return None;
        }

        let region = match self.region(self.vm_addr_start) {
            Ok(region) => region,
            Err(e) => {
                self.vm_addr_start = self.vm_addr_end;
                return Some(Err(e));
            }
        };

        let vm_addr = self.vm_addr_start;

        let chunk_len = if region.vm_addr_end <= self.vm_addr_end {
            // consume the whole region
            let len = region.vm_addr_end.saturating_sub(self.vm_addr_start);
            self.vm_addr_start = region.vm_addr_end;
            len
        } else {
            // consume part of the region
            let len = self.vm_addr_end.saturating_sub(self.vm_addr_start);
            self.vm_addr_start = self.vm_addr_end;
            len
        };

        Some(Ok((region, vm_addr, chunk_len as usize)))
    }
}

impl<'a> DoubleEndedIterator for MemoryChunkIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.vm_addr_start == self.vm_addr_end {
            return None;
        }

        let region = match self.region(self.vm_addr_end.saturating_sub(1)) {
            Ok(region) => region,
            Err(e) => {
                self.vm_addr_start = self.vm_addr_end;
                return Some(Err(e));
            }
        };

        let chunk_len = if region.vm_addr >= self.vm_addr_start {
            // consume the whole region
            let len = self.vm_addr_end.saturating_sub(region.vm_addr);
            self.vm_addr_end = region.vm_addr;
            len
        } else {
            // consume part of the region
            let len = self.vm_addr_end.saturating_sub(self.vm_addr_start);
            self.vm_addr_end = self.vm_addr_start;
            len
        };

        Some(Ok((region, self.vm_addr_end, chunk_len as usize)))
    }
}

pub fn is_nonoverlapping<N>(src: N, src_len: N, dst: N, dst_len: N) -> bool
where
    N: Ord + num_traits::SaturatingSub,
{
    // If the absolute distance between the ptrs is at least as big as the size of the other,
    // they do not overlap.
    if src > dst {
        src.saturating_sub(&dst) >= dst_len
    } else {
        dst.saturating_sub(&src) >= src_len
    }
}


// pub fn create_program_runtime_environment_v1<'a>(
//     reject_deployment_of_broken_elfs: bool,
// ) -> Result<BuiltinProgram<TestContextObject>, Error> {
//     // let enable_alt_bn128_syscall = feature_set.is_active(&enable_alt_bn128_syscall::id());
//     // let enable_alt_bn128_compression_syscall =
//         // feature_set.is_active(&enable_alt_bn128_compression_syscall::id());
//     // let enable_big_mod_exp_syscall = feature_set.is_active(&enable_big_mod_exp_syscall::id());
//     // let blake3_syscall_enabled = feature_set.is_active(&blake3_syscall_enabled::id());
//     // let curve25519_syscall_enabled = feature_set.is_active(&curve25519_syscall_enabled::id());
//     // let disable_fees_sysvar = feature_set.is_active(&disable_fees_sysvar::id());
//     // let epoch_rewards_syscall_enabled =
//         // feature_set.is_active(&enable_partitioned_epoch_reward::id());
//     // let disable_deploy_of_alloc_free_syscall = reject_deployment_of_broken_elfs
//     //     && feature_set.is_active(&disable_deploy_of_alloc_free_syscall::id());
//     // let last_restart_slot_syscall_enabled = feature_set.is_active(&last_restart_slot_sysvar::id());
//     // let enable_poseidon_syscall = feature_set.is_active(&enable_poseidon_syscall::id());
//     // let remaining_compute_units_syscall_enabled =
//         // feature_set.is_active(&remaining_compute_units_syscall_enabled::id());
//     // !!! ATTENTION !!!
//     // When adding new features for RBPF here,
//     // also add them to `Bank::apply_builtin_program_feature_transitions()`.

//     let config = Config {
//         max_call_depth: 20,
//         stack_frame_size: 4096,
//         enable_address_translation: true,
//         enable_stack_frame_gaps: false,
//         instruction_meter_checkpoint_distance: 10000,
//         enable_instruction_meter: true,
//         enable_instruction_tracing: true,
//         enable_symbol_and_section_labels: true,
//         reject_broken_elfs: reject_deployment_of_broken_elfs,
//         noop_instruction_rate: 256,
//         sanitize_user_provided_values: true,
//         external_internal_function_hash_collision: true,
//         reject_callx_r10: true,
//         enable_sbpf_v1: true,
//         enable_sbpf_v2: false,
//         optimize_rodata: false,
//         new_elf_parser: false,
//         aligned_memory_mapping: true,
//         // Warning, do not use `Config::default()` so that configuration here is explicit.
//     };
//     let mut result = FunctionRegistry::<BuiltinFunction<TestContextObject>>::default();

//     // Abort
//     result.register_function_hashed(*b"abort", SyscallAbort::vm)?;

//     // Panic
//     result.register_function_hashed(*b"sol_panic_", SyscallPanic::vm)?;

//     // Logging
//     result.register_function_hashed(*b"sol_log_", SyscallLog::vm)?;
//     // result.register_function_hashed(*b"sol_log_64_", SyscallLogU64::vm)?;
//     result.register_function_hashed(*b"sol_log_compute_units_", SyscallLogBpfComputeUnits::vm)?;
//     // result.register_function_hashed(*b"sol_log_pubkey", SyscallLogPubkey::vm)?;

//     // Program defined addresses (PDA)
//     // result.register_function_hashed(
//     //     *b"sol_create_program_address",
//     //     SyscallCreateProgramAddress::vm,
//     // )?;

//     // result.register_function_hashed(
//     //     *b"sol_try_find_program_address",
//     //     SyscallTryFindProgramAddress::vm,
//     // )?;

//     // Sha256
//     // result.register_function_hashed(*b"sol_sha256", SyscallHash::vm::<Sha256Hasher>)?;

//     // Keccak256
//     // result.register_function_hashed(*b"sol_keccak256", SyscallHash::vm::<Keccak256Hasher>)?;

//     // Secp256k1 Recover
//     // result.register_function_hashed(*b"sol_secp256k1_recover", SyscallSecp256k1Recover::vm)?;

//     // Blake3
//     // register_feature_gated_function!(
//     //     result,
//     //     blake3_syscall_enabled,
//     //     *b"sol_blake3",
//     //     SyscallHash::vm::<Blake3Hasher>,
//     // )?;

//     // Elliptic Curve Operations
//     // register_feature_gated_function!(
//     //     result,
//     //     curve25519_syscall_enabled,
//     //     *b"sol_curve_validate_point",
//     //     SyscallCurvePointValidation::vm,
//     // )?;
//     // register_feature_gated_function!(
//     //     result,
//     //     curve25519_syscall_enabled,
//     //     *b"sol_curve_group_op",
//     //     SyscallCurveGroupOps::vm,
//     // )?;
//     // register_feature_gated_function!(
//     //     result,
//     //     curve25519_syscall_enabled,
//     //     *b"sol_curve_multiscalar_mul",
//     //     SyscallCurveMultiscalarMultiplication::vm,
//     // )?;

//     // // Sysvars
//     // result.register_function_hashed(*b"sol_get_clock_sysvar", SyscallGetClockSysvar::vm)?;

//     // result.register_function_hashed(
//     //     *b"sol_get_epoch_schedule_sysvar",
//     //     SyscallGetEpochScheduleSysvar::vm,
//     // )?;

//     // register_feature_gated_function!(
//     //     result,
//     //     !disable_fees_sysvar,
//     //     *b"sol_get_fees_sysvar",
//     //     SyscallGetFeesSysvar::vm,
//     // )?;

//     // result.register_function_hashed(*b"sol_get_rent_sysvar", SyscallGetRentSysvar::vm)?;

//     // register_feature_gated_function!(
//     //     result,
//     //     last_restart_slot_syscall_enabled,
//     //     *b"sol_get_last_restart_slot",
//     //     SyscallGetLastRestartSlotSysvar::vm,
//     // )?;

//     // register_feature_gated_function!(
//     //     result,
//     //     epoch_rewards_syscall_enabled,
//     //     *b"sol_get_epoch_rewards_sysvar",
//     //     SyscallGetEpochRewardsSysvar::vm,
//     // )?;

//     // Memory ops
//     result.register_function_hashed(*b"sol_memcpy_", SyscallMemcpy::vm)?;
//     result.register_function_hashed(*b"sol_memmove_", SyscallMemmove::vm)?;
//     result.register_function_hashed(*b"sol_memcmp_", SyscallMemcmp::vm)?;
//     result.register_function_hashed(*b"sol_memset_", SyscallMemset::vm)?;

//     // Processed sibling instructions
//     // result.register_function_hashed(
//     //     *b"sol_get_processed_sibling_instruction",
//     //     SyscallGetProcessedSiblingInstruction::vm,
//     // )?;

//     // Stack height
//     // result.register_function_hashed(*b"sol_get_stack_height", SyscallGetStackHeight::vm)?;

//     // Return data
//     result.register_function_hashed(*b"sol_set_return_data", SyscallSetReturnData::vm)?;
//     // result.register_function_hashed(*b"sol_get_return_data", SyscallGetReturnData::vm)?;

//     // Cross-program invocation
//     // result.register_function_hashed(*b"sol_invoke_signed_c", SyscallInvokeSignedC::vm)?;
//     // result.register_function_hashed(*b"sol_invoke_signed_rust", SyscallInvokeSignedRust::vm)?;

//     // Memory allocator
//     // register_feature_gated_function!(
//     //     result,
//     //     true,
//     //     *b"sol_alloc_free_",
//     //     SyscallAllocFree::vm,
//     // )?;

//     // result.register_function_hashed( *b"sol_alloc_free_",SyscallAllocFree::vm)?;

//     // // Alt_bn128
//     // register_feature_gated_function!(
//     //     result,
//     //     enable_alt_bn128_syscall,
//     //     *b"sol_alt_bn128_group_op",
//     //     SyscallAltBn128::vm,
//     // )?;

//     // // Big_mod_exp
//     // register_feature_gated_function!(
//     //     result,
//     //     enable_big_mod_exp_syscall,
//     //     *b"sol_big_mod_exp",
//     //     SyscallBigModExp::vm,
//     // )?;

//     // // Poseidon
//     // register_feature_gated_function!(
//     //     result,
//     //     enable_poseidon_syscall,
//     //     *b"sol_poseidon",
//     //     SyscallPoseidon::vm,
//     // )?;

//     // // Accessing remaining compute units
//     // register_feature_gated_function!(
//     //     result,
//     //     remaining_compute_units_syscall_enabled,
//     //     *b"sol_remaining_compute_units",
//     //     SyscallRemainingComputeUnits::vm
//     // )?;

//     // // Alt_bn128_compression
//     // register_feature_gated_function!(
//     //     result,
//     //     enable_alt_bn128_compression_syscall,
//     //     *b"sol_alt_bn128_compression",
//     //     SyscallAltBn128Compression::vm,
//     // )?;

//     // Log data
//     // result.register_function_hashed(*b"sol_log_data", SyscallLogData::vm)?;

//     Ok(BuiltinProgram::new_loader(config, result))
// }


// fn address_is_aligned<T>(address: u64) -> bool {
//     (address as *mut T as usize)
//         .checked_rem(align_of::<T>())
//         .map(|rem| rem == 0)
//         .expect("T to be non-zero aligned")
// }

// fn translate(
//     memory_mapping: &MemoryMapping,
//     access_type: AccessType,
//     vm_addr: u64,
//     len: u64,
// ) -> Result<u64, Error> {
//     memory_mapping
//         .map(access_type, vm_addr, len)
//         .map_err(|err| err.into())
//         .into()
// }

// fn translate_type_inner<'a, T>(
//     memory_mapping: &MemoryMapping,
//     access_type: AccessType,
//     vm_addr: u64,
//     check_aligned: bool,
// ) -> Result<&'a mut T, Error> {
//     let host_addr = translate(memory_mapping, access_type, vm_addr, size_of::<T>() as u64)?;
//     if !check_aligned {
//         Ok(unsafe { std::mem::transmute::<u64, &mut T>(host_addr) })
//     } else if !address_is_aligned::<T>(host_addr) {
//         Err("UnalignedPointer".into())
//     } else {
//         Ok(unsafe { &mut *(host_addr as *mut T) })
//     }
// }
// fn translate_type_mut<'a, T>(
//     memory_mapping: &MemoryMapping,
//     vm_addr: u64,
//     check_aligned: bool,
// ) -> Result<&'a mut T, Error> {
//     translate_type_inner::<T>(memory_mapping, AccessType::Store, vm_addr, check_aligned)
// }
// fn translate_type<'a, T>(
//     memory_mapping: &MemoryMapping,
//     vm_addr: u64,
//     check_aligned: bool,
// ) -> Result<&'a T, Error> {
//     translate_type_inner::<T>(memory_mapping, AccessType::Load, vm_addr, check_aligned)
//         .map(|value| &*value)
// }

// fn translate_slice_inner<'a, T>(
//     memory_mapping: &MemoryMapping,
//     access_type: AccessType,
//     vm_addr: u64,
//     len: u64,
//     check_aligned: bool,
// ) -> Result<&'a mut [T], Error> {
//     if len == 0 {
//         return Ok(&mut []);
//     }

//     let total_size = len.saturating_mul(size_of::<T>() as u64);
//     if isize::try_from(total_size).is_err() {
//         return Err("InvalidLength".into());
//     }

//     let host_addr = translate(memory_mapping, access_type, vm_addr, total_size)?;

//     if check_aligned && !address_is_aligned::<T>(host_addr) {
//         return Err("UnalignedPointer".into());
//     }
//     Ok(unsafe { from_raw_parts_mut(host_addr as *mut T, len as usize) })
// }
// fn translate_slice_mut<'a, T>(
//     memory_mapping: &MemoryMapping,
//     vm_addr: u64,
//     len: u64,
//     check_aligned: bool,
// ) -> Result<&'a mut [T], Error> {
//     translate_slice_inner::<T>(
//         memory_mapping,
//         AccessType::Store,
//         vm_addr,
//         len,
//         check_aligned,
//     )
// }
// fn translate_slice<'a, T>(
//     memory_mapping: &MemoryMapping,
//     vm_addr: u64,
//     len: u64,
//     check_aligned: bool,
// ) -> Result<&'a [T], Error> {
//     translate_slice_inner::<T>(
//         memory_mapping,
//         AccessType::Load,
//         vm_addr,
//         len,
//         check_aligned,
//     )
//     .map(|value| &*value)
// }

// /// Take a virtual pointer to a string (points to SBF VM memory space), translate it
// /// pass it to a user-defined work function
// fn translate_string_and_do(
//     memory_mapping: &MemoryMapping,
//     addr: u64,
//     len: u64,
//     check_aligned: bool,
//     work: &mut dyn FnMut(&str) -> Result<u64, Error>,
// ) -> Result<u64, Error> {
//     let buf = translate_slice::<u8>(memory_mapping, addr, len, check_aligned)?;
//     match from_utf8(buf) {
//         Ok(message) => work(message),
//         Err(err) => Err(format!("InvalidString {:?} {:?}",err,buf.to_vec()).into()),
//     }
// }

// declare_builtin_function!(
//     /// Abort syscall functions, called when the SBF program calls `abort()`
//     /// LLVM will insert calls to `abort()` if it detects an untenable situation,
//     /// `abort()` is not intended to be called explicitly by the program.
//     /// Causes the SBF program to be halted immediately
//     SyscallAbort,
//     fn rust(
//         _invoke_context: &mut TestContextObject,
//         _arg1: u64,
//         _arg2: u64,
//         _arg3: u64,
//         _arg4: u64,
//         _arg5: u64,
//         _memory_mapping: &mut MemoryMapping,
//     ) -> Result<u64, Error> {
//         Err("memoryerror".into())
//     }
// );
// declare_builtin_function!(
//     /// Log current compute consumption
//     SyscallLogBpfComputeUnits,
//     fn rust(
//         invoke_context: &mut TestContextObject,
//         _arg1: u64,
//         _arg2: u64,
//         _arg3: u64,
//         _arg4: u64,
//         _arg5: u64,
//         _memory_mapping: &mut MemoryMapping,
//     ) -> Result<u64, Error> {
//         // let cost = invoke_context.get_compute_budget().syscall_base_cost;
//         // consume_compute_meter(invoke_context, cost)?;

//         // ic_logger_msg!(
//         //     invoke_context.get_log_collector(),
//         //     "Program consumption: {} units remaining",
//         //     invoke_context.get_remaining(),
//         // );
//         Ok(0)
//     }
// );
// declare_builtin_function!(
//     /// Set return data
//     SyscallSetReturnData,
//     fn rust(
//         invoke_context: &mut TestContextObject,
//         addr: u64,
//         len: u64,
//         _arg3: u64,
//         _arg4: u64,
//         _arg5: u64,
//         memory_mapping: &mut MemoryMapping,
//     ) -> Result<u64, Error> {

//         // if len > MAX_RETURN_DATA as u64 {
//         //     return Err(SyscallError::ReturnDataTooLarge(len, MAX_RETURN_DATA as u64).into());
//         // }

//         // let return_data = if len == 0 {
//         //     Vec::new()
//         // } else {
//         //     translate_slice::<u8>(
//         //         memory_mapping,
//         //         addr,
//         //         len,
//         //         // invoke_context.get_check_aligned(),
//         //         true
//         //     )?
//         //     .to_vec()
//         // };
//         // let transaction_context = &mut invoke_context.transaction_context;
//         // let program_id = *transaction_context
//         //     .get_current_instruction_context()
//         //     .and_then(|instruction_context| {
//         //         instruction_context.get_last_program_key(transaction_context)
//         //     })?;

//         // transaction_context.set_return_data(program_id, return_data)?;

//         Ok(0)
//     }
// );
// declare_builtin_function!(
//     /// Panic syscall function, called when the SBF program calls 'sol_panic_()`
//     /// Causes the SBF program to be halted immediately
//     SyscallPanic,
//     fn rust(
//         invoke_context: &mut TestContextObject,
//         file: u64,
//         len: u64,
//         line: u64,
//         column: u64,
//         _arg5: u64,
//         memory_mapping: &mut MemoryMapping,
//     ) -> Result<u64, Error> {

//         translate_string_and_do(
//             memory_mapping,
//             file,
//             len,
//             true,
//             &mut |string: &str| Err(
//                 format!("SyscallError::Panic {:?} {:?} {:?}", string.to_string(), line, column).into()),
//         )
//     }
// );

// declare_builtin_function!(
//     /// Log a user's info message
//     SyscallLog,
//     fn rust(
//         invoke_context: &mut TestContextObject,
//         addr: u64,
//         len: u64,
//         _arg3: u64,
//         _arg4: u64,
//         _arg5: u64,
//         memory_mapping: &mut MemoryMapping,
//     ) -> Result<u64, Error> {
//         let buf = translate_slice::<u8>(memory_mapping, addr, len, true)?;
//         match from_utf8(buf) {
//             Ok(message) => {
//                 log::debug!("Output ");
//                         Ok(0)
//             }
//             Err(err) => Err(format!("InvalidString {:?} {:?}",err,buf.to_vec()).into()),
//         }
//     }
// );


// // declare_builtin_function!(
//     /// Dynamic memory allocation syscall called when the SBF program calls
//     /// `sol_alloc_free_()`.  The allocator is expected to allocate/free
//     /// from/to a given chunk of memory and enforce size restrictions.  The
//     /// memory chunk is given to the allocator during allocator creation and
//     /// information about that memory (start address and size) is passed
//     /// to the VM to use for enforcement.
//     // SyscallAllocFree,
//     // fn rust(
//     //     invoke_context: &mut TestContextObject,
//     //     size: u64,
//     //     free_addr: u64,
//     //     _arg3: u64,
//     //     _arg4: u64,
//     //     _arg5: u64,
//     //     _memory_mapping: &mut MemoryMapping,
//     // ) -> Result<u64, Error> {
//     //     // TODO : HARDCODED to 8, change it 
//     //     let Ok(layout) = Layout::from_size_align(size as usize, 8) else {
//     //         return Ok(0);
//     //     };
//     //     let allocator = &mut invoke_context.get_syscall_context_mut()?.allocator;
//     //     if free_addr == 0 {
//     //         match allocator.alloc(layout) {
//     //             Ok(addr) => Ok(addr),
//     //             Err(_) => Ok(0),
//     //         }
//     //     } else {
//     //         // Unimplemented
//     //         Ok(0)
//     //     }
// //     }
// // );




// // fn mem_op_consume(invoke_context: &mut TestContextObject, n: u64) -> Result<(), Error> {
// //     let compute_budget = invoke_context.get_compute_budget();
// //     let cost = compute_budget.mem_op_base_cost.max(
// //         n.checked_div(compute_budget.cpi_bytes_per_unit)
// //             .unwrap_or(u64::MAX),
// //     );
// //     // consume_compute_meter(invoke_context, cost)
// // }

// declare_builtin_function!(
//     /// memcpy
//     SyscallMemcpy,
//     fn rust(
//         invoke_context: &mut TestContextObject,
//         dst_addr: u64,
//         src_addr: u64,
//         n: u64,
//         _arg4: u64,
//         _arg5: u64,
//         memory_mapping: &mut MemoryMapping,
//     ) -> Result<u64, Error> {
//         // mem_op_consume(invoke_context, n)?;

//         if !is_nonoverlapping(src_addr, n, dst_addr, n) {
//             return Err(format!("SyscallError::CopyOverlapping").into());
//         }

//         // host addresses can overlap so we always invoke memmove
//         memmove(invoke_context, dst_addr, src_addr, n, memory_mapping)
//     }
// );

// declare_builtin_function!(
//     /// memmove
//     SyscallMemmove,
//     fn rust(
//         invoke_context: &mut TestContextObject,
//         dst_addr: u64,
//         src_addr: u64,
//         n: u64,
//         _arg4: u64,
//         _arg5: u64,
//         memory_mapping: &mut MemoryMapping,
//     ) -> Result<u64, Error> {
//         // mem_op_consume(invoke_context, n)?;

//         memmove(invoke_context, dst_addr, src_addr, n, memory_mapping)
//     }
// );

// declare_builtin_function!(
//     /// memcmp
//     SyscallMemcmp,
//     fn rust(
//         invoke_context: &mut TestContextObject,
//         s1_addr: u64,
//         s2_addr: u64,
//         n: u64,
//         cmp_result_addr: u64,
//         _arg5: u64,
//         memory_mapping: &mut MemoryMapping,
//     ) -> Result<u64, Error> {
//         // mem_op_consume(invoke_context, n)?;

//             let cmp_result = translate_type_mut::<i32>(
//                 memory_mapping,
//                 cmp_result_addr,
//                 // invoke_context.get_check_aligned(),
//                 true,
//             )?;
//             *cmp_result = memcmp_non_contiguous(s1_addr, s2_addr, n, memory_mapping)?;
//         // } else {
//         //     let s1 = translate_slice::<u8>(
//         //         memory_mapping,
//         //         s1_addr,
//         //         n,
//         //         invoke_context.get_check_aligned(),
//         //     )?;
//         //     let s2 = translate_slice::<u8>(
//         //         memory_mapping,
//         //         s2_addr,
//         //         n,
//         //         invoke_context.get_check_aligned(),
//         //     )?;
//         //     let cmp_result = translate_type_mut::<i32>(
//         //         memory_mapping,
//         //         cmp_result_addr,
//         //         invoke_context.get_check_aligned(),
//         //     )?;

//         //     debug_assert_eq!(s1.len(), n as usize);
//         //     debug_assert_eq!(s2.len(), n as usize);
//         //     // Safety:
//         //     // memcmp is marked unsafe since it assumes that the inputs are at least
//         //     // `n` bytes long. `s1` and `s2` are guaranteed to be exactly `n` bytes
//         //     // long because `translate_slice` would have failed otherwise.
//         //     *cmp_result = unsafe { memcmp(s1, s2, n as usize) };
//         // }

//         Ok(0)
//     }
// );

// declare_builtin_function!(
//     /// memset
//     SyscallMemset,
//     fn rust(
//         invoke_context: &mut TestContextObject,
//         dst_addr: u64,
//         c: u64,
//         n: u64,
//         _arg4: u64,
//         _arg5: u64,
//         memory_mapping: &mut MemoryMapping,
//     ) -> Result<u64, Error> {
//         // mem_op_consume(invoke_context, n)?;

//         // if invoke_context
//         //     .feature_set
//         //     .is_active(&feature_set::bpf_account_data_direct_mapping::id())
//         // {
//             memset_non_contiguous(dst_addr, c as u8, n, memory_mapping)
//         // } else {
//         //     let s = translate_slice_mut::<u8>(
//         //         memory_mapping,
//         //         dst_addr,
//         //         n,
//         //         invoke_context.get_check_aligned(),
//         //     )?;
//         //     s.fill(c as u8);
//         //     Ok(0)
//         // }
//     }
// );

// #[macro_export]
// macro_rules! register_feature_gated_function {
//     ($result:expr, $is_feature_active:expr, $name:expr, $call:expr $(,)?) => {
//         if $is_feature_active {
//             $result.register_function_hashed($name, $call)
//         } else {
//             Ok(0)
//         }
//     };
// }

// fn memmove(
//     invoke_context: &mut TestContextObject,
//     dst_addr: u64,
//     src_addr: u64,
//     n: u64,
//     memory_mapping: &MemoryMapping,
// ) -> Result<u64, Error> {
//     // if invoke_context
//     //     .feature_set
//     //     .is_active(&feature_set::bpf_account_data_direct_mapping::id())
//     // {
//         memmove_non_contiguous(dst_addr, src_addr, n, memory_mapping)
//     // } else {
//     //     let dst_ptr = translate_slice_mut::<u8>(
//     //         memory_mapping,
//     //         dst_addr,
//     //         n,
//     //         invoke_context.get_check_aligned(),
//     //     )?
//     //     .as_mut_ptr();
//     //     let src_ptr = translate_slice::<u8>(
//     //         memory_mapping,
//     //         src_addr,
//     //         n,
//     //         invoke_context.get_check_aligned(),
//     //     )?
//     //     .as_ptr();

//     //     unsafe { std::ptr::copy(src_ptr, dst_ptr, n as usize) };
//     //     Ok(0)
//     // }
// }

// fn memmove_non_contiguous(
//     dst_addr: u64,
//     src_addr: u64,
//     n: u64,
//     memory_mapping: &MemoryMapping,
// ) -> Result<u64, Error> {
//     let reverse = dst_addr.wrapping_sub(src_addr) < n;
//     iter_memory_pair_chunks(
//         AccessType::Load,
//         src_addr,
//         AccessType::Store,
//         dst_addr,
//         n,
//         memory_mapping,
//         reverse,
//         |src_host_addr, dst_host_addr, chunk_len| {
//             unsafe { std::ptr::copy(src_host_addr, dst_host_addr as *mut u8, chunk_len) };
//             Ok(0)
//         },
//     )
// }

// // Marked unsafe since it assumes that the slices are at least `n` bytes long.
// unsafe fn memcmp(s1: &[u8], s2: &[u8], n: usize) -> i32 {
//     for i in 0..n {
//         let a = *s1.get_unchecked(i);
//         let b = *s2.get_unchecked(i);
//         if a != b {
//             return (a as i32).saturating_sub(b as i32);
//         };
//     }

//     0
// }

// fn memcmp_non_contiguous(
//     src_addr: u64,
//     dst_addr: u64,
//     n: u64,
//     memory_mapping: &MemoryMapping,
// ) -> Result<i32, Error> {
//     let memcmp_chunk = |s1_addr, s2_addr, chunk_len| {
//         let res = unsafe {
//             let s1 = slice::from_raw_parts(s1_addr, chunk_len);
//             let s2 = slice::from_raw_parts(s2_addr, chunk_len);
//             // Safety:
//             // memcmp is marked unsafe since it assumes that s1 and s2 are exactly chunk_len
//             // long. The whole point of iter_memory_pair_chunks is to find same length chunks
//             // across two memory regions.
//             memcmp(s1, s2, chunk_len)
//         };
//         if res != 0 {
//             return Err(MemcmpError::Diff(res).into());
//         }
//         Ok(0)
//     };
//     match iter_memory_pair_chunks(
//         AccessType::Load,
//         src_addr,
//         AccessType::Load,
//         dst_addr,
//         n,
//         memory_mapping,
//         false,
//         memcmp_chunk,
//     ) {
//         Ok(res) => Ok(res),
//         Err(error) => match error.downcast_ref() {
//             Some(MemcmpError::Diff(diff)) => Ok(*diff),
//             _ => Err(error),
//         },
//     }
// }

// #[derive(Debug)]
// enum MemcmpError {
//     Diff(i32),
// }

// impl std::fmt::Display for MemcmpError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             MemcmpError::Diff(diff) => write!(f, "memcmp diff: {diff}"),
//         }
//     }
// }

// impl std::error::Error for MemcmpError {
//     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//         match self {
//             MemcmpError::Diff(_) => None,
//         }
//     }
// }

// fn memset_non_contiguous(
//     dst_addr: u64,
//     c: u8,
//     n: u64,
//     memory_mapping: &MemoryMapping,
// ) -> Result<u64, Error> {
//     let dst_chunk_iter = MemoryChunkIterator::new(memory_mapping, AccessType::Store, dst_addr, n)?;
//     for item in dst_chunk_iter {
//         let (dst_region, dst_vm_addr, dst_len) = item?;
//         let dst_host_addr = Result::from(dst_region.vm_to_host(dst_vm_addr, dst_len as u64))?;
//         unsafe { slice::from_raw_parts_mut(dst_host_addr as *mut u8, dst_len).fill(c) }
//     }

//     Ok(0)
// }

// fn iter_memory_pair_chunks<T, F>(
//     src_access: AccessType,
//     src_addr: u64,
//     dst_access: AccessType,
//     dst_addr: u64,
//     n_bytes: u64,
//     memory_mapping: &MemoryMapping,
//     reverse: bool,
//     mut fun: F,
// ) -> Result<T, Error>
// where
//     T: Default,
//     F: FnMut(*const u8, *const u8, usize) -> Result<T, Error>,
// {
//     let mut src_chunk_iter =
//         MemoryChunkIterator::new(memory_mapping, src_access, src_addr, n_bytes)
//             .map_err(EbpfError::from)?;
//     let mut dst_chunk_iter =
//         MemoryChunkIterator::new(memory_mapping, dst_access, dst_addr, n_bytes)
//             .map_err(EbpfError::from)?;

//     let mut src_chunk = None;
//     let mut dst_chunk = None;

//     macro_rules! memory_chunk {
//         ($chunk_iter:ident, $chunk:ident) => {
//             if let Some($chunk) = &mut $chunk {
//                 // Keep processing the current chunk
//                 $chunk
//             } else {
//                 // This is either the first call or we've processed all the bytes in the current
//                 // chunk. Move to the next one.
//                 let chunk = match if reverse {
//                     $chunk_iter.next_back()
//                 } else {
//                     $chunk_iter.next()
//                 } {
//                     Some(item) => item?,
//                     None => break,
//                 };
//                 $chunk.insert(chunk)
//             }
//         };
//     }

//     loop {
//         let (src_region, src_chunk_addr, src_remaining) = memory_chunk!(src_chunk_iter, src_chunk);
//         let (dst_region, dst_chunk_addr, dst_remaining) = memory_chunk!(dst_chunk_iter, dst_chunk);

//         // We always process same-length pairs
//         let chunk_len = *src_remaining.min(dst_remaining);

//         let (src_host_addr, dst_host_addr) = {
//             let (src_addr, dst_addr) = if reverse {
//                 // When scanning backwards not only we want to scan regions from the end,
//                 // we want to process the memory within regions backwards as well.
//                 (
//                     src_chunk_addr
//                         .saturating_add(*src_remaining as u64)
//                         .saturating_sub(chunk_len as u64),
//                     dst_chunk_addr
//                         .saturating_add(*dst_remaining as u64)
//                         .saturating_sub(chunk_len as u64),
//                 )
//             } else {
//                 (*src_chunk_addr, *dst_chunk_addr)
//             };

//             (
//                 Result::from(src_region.vm_to_host(src_addr, chunk_len as u64))?,
//                 Result::from(dst_region.vm_to_host(dst_addr, chunk_len as u64))?,
//             )
//         };

//         fun(
//             src_host_addr as *const u8,
//             dst_host_addr as *const u8,
//             chunk_len,
//         )?;

//         // Update how many bytes we have left to scan in each chunk
//         *src_remaining = src_remaining.saturating_sub(chunk_len);
//         *dst_remaining = dst_remaining.saturating_sub(chunk_len);

//         if !reverse {
//             // We've scanned `chunk_len` bytes so we move the vm address forward. In reverse
//             // mode we don't do this since we make progress by decreasing src_len and
//             // dst_len.
//             *src_chunk_addr = src_chunk_addr.saturating_add(chunk_len as u64);
//             *dst_chunk_addr = dst_chunk_addr.saturating_add(chunk_len as u64);
//         }

//         if *src_remaining == 0 {
//             src_chunk = None;
//         }

//         if *dst_remaining == 0 {
//             dst_chunk = None;
//         }
//     }

//     Ok(T::default())
// }

// struct MemoryChunkIterator<'a> {
//     memory_mapping: &'a MemoryMapping<'a>,
//     access_type: AccessType,
//     initial_vm_addr: u64,
//     vm_addr_start: u64,
//     // exclusive end index (start + len, so one past the last valid address)
//     vm_addr_end: u64,
//     len: u64,
// }

// impl<'a> MemoryChunkIterator<'a> {
//     fn new(
//         memory_mapping: &'a MemoryMapping,
//         access_type: AccessType,
//         vm_addr: u64,
//         len: u64,
//     ) -> Result<MemoryChunkIterator<'a>, EbpfError> {
//         let vm_addr_end = vm_addr.checked_add(len).ok_or(EbpfError::AccessViolation(
//             access_type,
//             vm_addr,
//             len,
//             "unknown",
//         ))?;
//         Ok(MemoryChunkIterator {
//             memory_mapping,
//             access_type,
//             initial_vm_addr: vm_addr,
//             len,
//             vm_addr_start: vm_addr,
//             vm_addr_end,
//         })
//     }

//     fn region(&mut self, vm_addr: u64) -> Result<&'a MemoryRegion, Error> {
//         match self.memory_mapping.region(self.access_type, vm_addr) {
//             Ok(region) => Ok(region),
//             Err(error) => match error {
//                 EbpfError::AccessViolation(access_type, _vm_addr, _len, name) => Err(Box::new(
//                     EbpfError::AccessViolation(access_type, self.initial_vm_addr, self.len, name),
//                 )),
//                 EbpfError::StackAccessViolation(access_type, _vm_addr, _len, frame) => {
//                     Err(Box::new(EbpfError::StackAccessViolation(
//                         access_type,
//                         self.initial_vm_addr,
//                         self.len,
//                         frame,
//                     )))
//                 }
//                 _ => Err(error.into()),
//             },
//         }
//     }
// }

// impl<'a> Iterator for MemoryChunkIterator<'a> {
//     type Item = Result<(&'a MemoryRegion, u64, usize), Error>;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.vm_addr_start == self.vm_addr_end {
//             return None;
//         }

//         let region = match self.region(self.vm_addr_start) {
//             Ok(region) => region,
//             Err(e) => {
//                 self.vm_addr_start = self.vm_addr_end;
//                 return Some(Err(e));
//             }
//         };

//         let vm_addr = self.vm_addr_start;

//         let chunk_len = if region.vm_addr_end <= self.vm_addr_end {
//             // consume the whole region
//             let len = region.vm_addr_end.saturating_sub(self.vm_addr_start);
//             self.vm_addr_start = region.vm_addr_end;
//             len
//         } else {
//             // consume part of the region
//             let len = self.vm_addr_end.saturating_sub(self.vm_addr_start);
//             self.vm_addr_start = self.vm_addr_end;
//             len
//         };

//         Some(Ok((region, vm_addr, chunk_len as usize)))
//     }
// }

// impl<'a> DoubleEndedIterator for MemoryChunkIterator<'a> {
//     fn next_back(&mut self) -> Option<Self::Item> {
//         if self.vm_addr_start == self.vm_addr_end {
//             return None;
//         }

//         let region = match self.region(self.vm_addr_end.saturating_sub(1)) {
//             Ok(region) => region,
//             Err(e) => {
//                 self.vm_addr_start = self.vm_addr_end;
//                 return Some(Err(e));
//             }
//         };

//         let chunk_len = if region.vm_addr >= self.vm_addr_start {
//             // consume the whole region
//             let len = self.vm_addr_end.saturating_sub(region.vm_addr);
//             self.vm_addr_end = region.vm_addr;
//             len
//         } else {
//             // consume part of the region
//             let len = self.vm_addr_end.saturating_sub(self.vm_addr_start);
//             self.vm_addr_end = self.vm_addr_start;
//             len
//         };

//         Some(Ok((region, self.vm_addr_end, chunk_len as usize)))
//     }
// }

// pub fn is_nonoverlapping<N>(src: N, src_len: N, dst: N, dst_len: N) -> bool
// where
//     N: Ord + num_traits::SaturatingSub,
// {
//     // If the absolute distance between the ptrs is at least as big as the size of the other,
//     // they do not overlap.
//     if src > dst {
//         src.saturating_sub(&dst) >= dst_len
//     } else {
//         dst.saturating_sub(&src) >= src_len
//     }
// }