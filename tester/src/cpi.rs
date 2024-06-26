
// use core_types::types::{Instruction, Pubkey, UtxoInfo, UtxoMeta};
// use solana_rbpf::memory_region::MemoryMapping;
// type Error = Box<dyn std::error::Error>;
// use crate::{config::SyscallInvokeSigned, processor::InvokeContext};

// pub fn invoke_signed_unchecked(
//     pubkey: &Pubkey,
//     utxos: &Vec<UtxoInfo>,
//     instruction_data : &Vec<u8>
// ) -> ProgramResult {
//     {
//         let instruction = CpiContext::from(pubkey.clone(),&*utxos.clone(),instruction_data.clone());
//         let result = unsafe {
//             crate::syscalls::sol_invoke_signed_rust(
//                 &instruction as *const _ as *const u8,
//             )
//         };
//         match result {
//             0 => Ok(()),
//             // crate::entrypoint::SUCCESS => Ok(()),
//             _ => Err(result.into()),
//         }
//     }
// }

// #[repr(C)]
// pub struct CpiContext<'a> {
//     pubkey: Pubkey,
//     utxos : &'a [UtxoInfo],
//     instruction_data: Vec<u8>
// }

// impl<'a> CpiContext<'a> {
//     pub fn from(pubkey : Pubkey, utxos : &Vec<UtxoInfo>, instruction_data: Vec<u8>) -> Self {
//         Self {
//             pubkey,
//             utxos,
//             instruction_data
//         }
//     }
// }

// pub fn cpi_common<S>(invoke_context : &mut InvokeContext, cpi_context_addr : u64, memory_mapping:& mut MemoryMapping) -> Result<u64, Error> 
// where S : SyscallInvokeSigned{
//     let context = S::translate_instruction(cpi_context_addr, memory_mapping, invoke_context)?;

//     // update the authorities
//     let transaction_context = invoke_context.get_current_transaction_context();
//     let utxo_data = transaction_context.get_data();
//     let utxo_authority = transaction_context.get_authorities();
//     let current_instruction_context = transaction_context.get_current_instruction_context();
//     let caller_program_id = current_instruction_context.get_program_id();
//     let utxos = context.utxos;

//     // for every utxo, update data and then check if the authoirty has been changed, if it has, change it to updated value

//     for utxo in utxos {
//         if let Some(caller_program_id) = utxo_authority.get(&utxo.id()) {
//             // update data 
//             *utxo_data.get_mut(&utxo.id()).expect("utox id must exist") = *utxo.data.borrow();
//             *utxo_authority.get_mut(&utxo.id()).expect("utox id must exist") = (*utxo.authority.borrow().0).to_vec();
//         }
//     }
//     // make new context
//     invoke_context.
//     Ok(0)
// }