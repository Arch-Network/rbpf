use core_types::{
    entrypoint::ProgramResult,
    types::{ProgramInstruction, StableInstruction},
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
        core_types::entrypoint::SUCCESS => Ok(()),
        _ => Err(result.into()),
    }
}
