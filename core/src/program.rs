use crate::{
    entrypoint::ProgramResult,
    types::{ProgramInstruction, Pubkey, StableInstruction},
    utxo_info::UtxoInfo,
};

pub fn invoke(
    instruction: &ProgramInstruction,
    account_infos: &[UtxoInfo],
) -> ProgramResult {
    let instruction = StableInstruction::from(instruction.clone());
    let result = unsafe {
        crate::syscalls::sol_invoke_signed_rust(
            &instruction as *const _ as *const u8,
            account_infos as *const _ as *const u8,
            account_infos.len() as u64,
        )
    };
    match result {
        crate::entrypoint::SUCCESS => Ok(()),
        _ => Err(result.into()),
    }
}

/// Maximum size that can be set using [`set_return_data`].
pub const MAX_RETURN_DATA: usize = 1024;


/// Set the running program's return data.
///
/// Return data is a dedicated per-transaction buffer for data passed
/// from cross-program invoked programs back to their caller.
///
/// The maximum size of return data is [`MAX_RETURN_DATA`]. Return data is
/// retrieved by the caller with [`get_return_data`].
pub fn set_return_data(data: &[u8]) {
    unsafe {
        crate::syscalls::sol_set_return_data(data.as_ptr(), data.len() as u64)
    };
}

/// Get the return data from an invoked program.
///
/// For every transaction there is a single buffer with maximum length
/// [`MAX_RETURN_DATA`], paired with a [`Pubkey`] representing the program ID of
/// the program that most recently set the return data. Thus the return data is
/// a global resource and care must be taken to ensure that it represents what
/// is expected: called programs are free to set or not set the return data; and
/// the return data may represent values set by programs multiple calls down the
/// call stack, depending on the circumstances of transaction execution.
///
/// Return data is set by the callee with [`set_return_data`].
///
/// Return data is cleared before every CPI invocation &mdash; a program that
/// has invoked no other programs can expect the return data to be `None`; if no
/// return data was set by the previous CPI invocation, then this function
/// returns `None`.
///
/// Return data is not cleared after returning from CPI invocations &mdash; a
/// program that has called another program may retrieve return data that was
/// not set by the called program, but instead set by a program further down the
/// call stack; or, if a program calls itself recursively, it is possible that
/// the return data was not set by the immediate call to that program, but by a
/// subsequent recursive call to that program. Likewise, an external RPC caller
/// may see return data that was not set by the program it is directly calling,
/// but by a program that program called.
///
/// For more about return data see the [documentation for the return data proposal][rdp].
///
/// [rdp]: https://docs.solanalabs.com/proposals/return-data
pub fn get_return_data() -> Option<(Pubkey, Vec<u8>)> {
        use std::cmp::min;

        let mut buf = [0u8; MAX_RETURN_DATA];
        let mut program_id = Pubkey::default();

        let size = unsafe {
            crate::syscalls::sol_get_return_data(
                buf.as_mut_ptr(),
                buf.len() as u64,
                &mut program_id,
            )
        };

        if size == 0 {
            None
        } else {
            let size = min(size as usize, MAX_RETURN_DATA);
            Some((program_id, buf[..size as usize].to_vec()))
        }
    
}