use borsh::from_slice;
use std::{
    alloc::Layout,
    cell::RefCell,
    collections::HashMap,
    mem::size_of,
    ptr::null_mut,
    rc::Rc,
    slice::{from_raw_parts, from_raw_parts_mut},
};
extern crate alloc;
use crate::{program_error::ProgramError, types::*, utxo_info::UtxoInfo};
use alloc::vec::Vec;
/// Start address of the memory region used for program heap.
pub const HEAP_START_ADDRESS: u64 = 0x300000000;
/// Length of the heap memory region used for program heap.
pub const HEAP_LENGTH: usize = 32 * 1024;
/// Maximum permitted size of account data (10 MiB).
pub const MAX_PERMITTED_DATA_LENGTH: u64 = 10 * 1024 * 1024;
/// Maximum number of bytes a program may add to an account during a single realloc
pub const MAX_PERMITTED_DATA_INCREASE: usize = 1_024 * 10;

pub const BPF_ALIGN_OF_U128: usize = 8;

/// Maximum number of instruction utxos that can be serialized into the
/// SBF VM.
pub const NON_DUP_MARKER: u8 = u8::MAX;

pub type ProgramResult = Result<(), ProgramError>;

/// Programs indicate success with a return value of 0
pub const SUCCESS: u64 = 0;

/// The bump allocator used as the default rust heap when running programs.
pub struct BumpAllocator {
    pub start: usize,
    pub len: usize,
}
/// Integer arithmetic in this global allocator implementation is safe when
/// operating on the prescribed `HEAP_START_ADDRESS` and `HEAP_LENGTH`. Any
/// other use may overflow and is thus unsupported and at one's own risk.
#[allow(clippy::arithmetic_side_effects)]
unsafe impl std::alloc::GlobalAlloc for BumpAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pos_ptr = self.start as *mut usize;

        let mut pos = *pos_ptr;
        if pos == 0 {
            // First time, set starting position
            pos = self.start + self.len;
        }
        pos = pos.saturating_sub(layout.size());
        pos &= !(layout.align().wrapping_sub(1));
        if pos < self.start + size_of::<*mut u8>() {
            return null_mut();
        }
        *pos_ptr = pos;
        pos as *mut u8
    }
    #[inline]
    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
        // I'm a bump allocator, I don't free
    }
}

pub unsafe fn deserialize<'a>(input: *mut u8) -> (&'a Pubkey, Vec<UtxoInfo<'a>>, &'a [u8]) {
    let mut offset: usize = 0;

    #[allow(clippy::cast_ptr_alignment)]
    let num_utxos = *(input.add(offset) as *const u64) as usize;
    offset += size_of::<u64>();

    let mut utxos = Vec::with_capacity(num_utxos);

    for _ in 0..num_utxos {
        let dup_info = *(input.add(offset) as *const u8);
        offset += size_of::<u8>();
        if dup_info == NON_DUP_MARKER {
            offset += 3 * size_of::<u8>();

            let vout = *(input.add(offset) as *const u32);
            offset += size_of::<u32>();

            let txid = &*(input.add(offset) as *const [u8; 32]);
            offset += size_of::<[u8; 32]>();

            //skipping original data_len
            offset += size_of::<u64>();

            let data_len = *(input.add(offset) as *const u64) as usize;
            offset += size_of::<u64>();

            let data = Rc::new(RefCell::new({
                from_raw_parts_mut(input.add(offset), data_len)
            }));

            offset += data_len + MAX_PERMITTED_DATA_INCREASE;
            offset += (offset as *const u8).align_offset(BPF_ALIGN_OF_U128); // padding

            let authority: &Pubkey = &*(input.add(offset) as *const Pubkey);
            offset += size_of::<Pubkey>();

            utxos.push(UtxoInfo {
                data,
                authority,
                txid,
                vout,
            });
        } else {
            offset += 7; // padding

            // Duplicate account, clone the original
            utxos.push(utxos[dup_info as usize].clone());
        }
    }

    // Instruction data

    #[allow(clippy::cast_ptr_alignment)]
    let instruction_data_len = *(input.add(offset) as *const u64) as usize;
    offset += size_of::<u64>();

    let instruction_data = { from_raw_parts(input.add(offset), instruction_data_len) };
    offset += instruction_data_len;

    // Program Id

    let program_id: &Pubkey = &*(input.add(offset) as *const Pubkey);

    (program_id, utxos, instruction_data)
}

#[macro_export]
macro_rules! entrypoint {
    ($process_instruction:ident) => {
        /// # Safety
        #[no_mangle]
        pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
            use std::collections::HashMap;
            let (program_id, utxos, instruction_data) =
                unsafe { $crate::entrypoint::deserialize(input) };
            match $process_instruction(&program_id, &utxos, &instruction_data) {
                Ok(()) => {
                    return 0;
                }
                Err(e) => {
                    return 1;
                }
            }
        }
        $crate::custom_heap_default!();
        // $crate::custom_panic_default!();
    };
}

#[macro_export]
macro_rules! custom_heap_default {
    () => {
        #[global_allocator]
        static A: $crate::entrypoint::BumpAllocator = $crate::entrypoint::BumpAllocator {
            start: $crate::entrypoint::HEAP_START_ADDRESS as usize,
            len: $crate::entrypoint::HEAP_LENGTH,
        };
    };
}

macro_rules! define_syscall {
	(fn $name:ident($($arg:ident: $typ:ty),*) -> $ret:ty) => {
		extern "C" {
			pub fn $name($($arg: $typ),*) -> $ret;
		}
	};
	(fn $name:ident($($arg:ident: $typ:ty),*)) => {
		define_syscall!(fn $name($($arg: $typ),*) -> ());
	}
}

define_syscall!(fn sol_log_(message: *const u8, len: u64));

mod Test {
    use super::*;

    //     #[test]
    //     fn test_entrypoint() {
    //         // #[global_allocator]
    //         // static A: BumpAllocator = BumpAllocator {
    //         //     start: HEAP_START_ADDRESS as usize,
    //         //     len: HEAP_LENGTH,
    //         // };

    //         let mut mem = construct_data();
    //         println!("input data len {}", mem.len());
    //         unsafe {entrypoint(mem.as_mut_ptr()); }

    //         let size = unsafe { *(mem.as_mut_ptr() as *mut u32)};

    //         println!("final {:?}", borsh::from_slice::<(HashMap<String,Vec<u8>>, HashMap<String,Vec<u8>>,Transaction)>(&mem[4..size as usize + 4 ]));

    //         }

    //     pub fn process_instruction(program_id: Pubkey, utxos : &[UtxoInfo], instruction_data : &Vec<u8>) -> Result<Transaction,String> {

    //         let txin = TxIn {
    //             txid: String::from("abcdef"),
    //             vout:1,
    //             script_sig: [12u8;32].to_vec(),
    //             sequence: 5,
    //             witness: vec![[12u8;32].to_vec(), [22u8;34].to_vec()]
    //        };

    //        let txout = TxOut {
    //            amount: 10240,
    //            script_pubkey: [122u8;64].to_vec(),
    //        };

    //        return Ok(Transaction {
    //            version: 1,
    //            input: vec![txin],
    //            output: vec![txout],
    //            lock_time: 15,
    //        })
    //     }

    //     pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
    //         use std::collections::HashMap;
    //         let (program_id, utxos, instruction_data) =
    //             unsafe { deserialize(input) };
    //         match process_instruction(program_id, &utxos, &instruction_data) {
    //             Ok(tx_hex) => {
    //                 let mut new_authorities: HashMap<String, Vec<u8>> = HashMap::new();
    //                 let mut new_data: HashMap<String, Vec<u8>> = HashMap::new();
    //                 utxos.iter().for_each(|utxo| {
    //                     new_authorities.insert(utxo.id(), utxo.authority.clone().into_inner().0);
    //                     new_data.insert(utxo.id(), utxo.data.clone().into_inner());
    //                 });
    //                let mut serialised_output = borsh::to_vec(&(new_authorities, new_data, tx_hex)).unwrap();
    //                 let output_len = serialised_output.len();
    //                 println!("output length is {}", output_len);
    //                 unsafe {*(input as *mut u32) = output_len as u32;}

    //                 unsafe {
    //                     std::ptr::copy_nonoverlapping(serialised_output.as_mut_ptr(),input.add(4),output_len);
    //                 }
    //                 return 0;
    //             }
    //             Err(e) => {
    //                 return 1;
    //             }
    //         }
    // }
}
