use borsh::from_slice;
use std::{
    alloc::Layout, cell::RefCell, collections::HashMap, mem::{self, size_of}, ptr::null_mut, rc::Rc,
};
extern crate alloc;
use crate::types::*;
use alloc::vec::Vec;
/// Start address of the memory region used for program heap.
pub const HEAP_START_ADDRESS: u64 = 0x300000000;
/// Length of the heap memory region used for program heap.
pub const HEAP_LENGTH: usize = 32 * 1024;

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

pub fn deserialize<'a>(input: *mut u8) -> (Pubkey, &'a [UtxoInfo], Vec<u8>) {
    let mut size = 0;
    let size_of_usize = mem::size_of::<usize>();
    let utxos_ptr = unsafe {
        *(input as *mut usize)
    };

    size += size_of_usize;

    let len = unsafe {
        *(input.add(size) as *const usize)
    };

    size += size_of_usize;

    let utxos = unsafe {
        std::slice::from_raw_parts(utxos_ptr as *const UtxoInfo, len)
    };
    let length_of_data = unsafe { *(input.add(size) as *mut u32) };

    size += mem::size_of::<u32>();

    let data_slice = unsafe { std::slice::from_raw_parts(input.add(size), length_of_data as usize) };

    let (program_id, instruction_data) = from_slice(data_slice)
        .expect("unable to deserialise input to entrypoint function");

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
            match $process_instruction(program_id, &utxos, instruction_data) {
                Ok(tx_hex) => {
                    // let mut new_authorities: HashMap<String, Vec<u8>> = HashMap::new();
                    // let mut new_data: HashMap<String, Vec<u8>> = HashMap::new();
                    // utxos.iter().for_each(|utxo| {
                    //     new_authorities.insert(utxo.id(), utxo.authority.clone().into_inner().0);
                    //     new_data.insert(utxo.id(), utxo.data.clone().into_inner());
                    // });
                    let mut serialised_output = borsh::to_vec(&(utxos, tx_hex)).unwrap();
                    let output_len = serialised_output.len();
                    unsafe {
                        *(input as *mut u32) = output_len as u32;
                    }

                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            serialised_output.as_mut_ptr(),
                            input.add(4),
                            output_len,
                        );
                    }
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

// #[macro_export]
// macro_rules! custom_panic_default {
//     () => {
//         #[no_mangle]
//         fn custom_panic(info: &core::panic::PanicInfo<'_>) {
//             // Full panic reporting
//             msg!("{}",info);
//         }
//     };
// }

// #[macro_export]
// macro_rules! msg {
//     ($msg:expr) => {
//         $crate::entrypoint::sol_log($msg)
//     };
//     ($($arg:tt)*) => ($crate::entrypoint::sol_log(&format!($($arg)*)));
// }

// /// Print a string to the log.
// #[inline]
// pub fn sol_log(message: &str) {
//     unsafe {
//         crate::entrypoint::sol_log_(message.as_ptr(), message.len() as u64);
//     }
// }
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

mod Test {
    use super::*;

    #[test]
    fn test_entrypoint() {
        // #[global_allocator]
        // static A: BumpAllocator = BumpAllocator {
        //     start: HEAP_START_ADDRESS as usize,
        //     len: HEAP_LENGTH,
        // };

        let mut mem = construct_data();
        println!("input data len {}", mem.len());
        unsafe {
            entrypoint(mem.as_mut_ptr());
        }

        let size = unsafe { *(mem.as_mut_ptr() as *mut u32) };

        println!(
            "final {:?}",
            borsh::from_slice::<(
                HashMap<String, Vec<u8>>,
                HashMap<String, Vec<u8>>,
                Transaction
            )>(&mem[4..size as usize + 4])
        );
    }

    pub fn process_instruction(
        program_id: Pubkey,
        utxos: &[UtxoInfo],
        instruction_data: &Vec<u8>,
    ) -> Result<Transaction, String> {
        let txin = TxIn {
            txid: String::from("abcdef"),
            vout: 1,
            script_sig: [12u8; 32].to_vec(),
            sequence: 5,
            witness: vec![[12u8; 32].to_vec(), [22u8; 34].to_vec()],
        };

        let txout = TxOut {
            amount: 10240,
            script_pubkey: [122u8; 64].to_vec(),
        };

        return Ok(Transaction {
            version: 1,
            input: vec![txin],
            output: vec![txout],
            lock_time: 15,
        });
    }

    pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
        use std::collections::HashMap;
        let (program_id, utxos, instruction_data) = unsafe { deserialize(input) };
        match process_instruction(program_id, &utxos, &instruction_data) {
            Ok(tx_hex) => {
                let mut new_authorities: HashMap<String, Vec<u8>> = HashMap::new();
                let mut new_data: HashMap<String, Vec<u8>> = HashMap::new();
                utxos.iter().for_each(|utxo| {
                    new_authorities.insert(utxo.id(), utxo.authority.clone().into_inner().0);
                    new_data.insert(utxo.id(), utxo.data.clone().into_inner());
                });
                let mut serialised_output =
                    borsh::to_vec(&(new_authorities, new_data, tx_hex)).unwrap();
                let output_len = serialised_output.len();
                println!("output length is {}", output_len);
                unsafe {
                    *(input as *mut u32) = output_len as u32;
                }

                unsafe {
                    std::ptr::copy_nonoverlapping(
                        serialised_output.as_mut_ptr(),
                        input.add(4),
                        output_len,
                    );
                }
                return 0;
            }
            Err(e) => {
                return 1;
            }
        }
    }

    fn construct_data() -> Vec<u8> {
        let utxo = UtxoMeta {
            txid: String::from("a"),
            vout: 2,
        };
        let instruction = Instruction {
            program_id: Pubkey([1u8; 32].to_vec()),
            utxos: vec![utxo.clone()],
            data: [0, 1, 0, 0, 0, 0, 0, 0, 0].to_vec(),
        };
        let mut authority = HashMap::new();
        authority.insert(String::from("a:2"), [12u8; 78].to_vec());

        let mut data: HashMap<String, Vec<u8>> = HashMap::new();
        data.insert(String::from("a:2"), [12u8; 78].to_vec());

        let input_to_entrypoint = (instruction, authority, data);
        let mut serialised_data = borsh::to_vec(&input_to_entrypoint).unwrap();
        let mut data_len = serialised_data.len() as u32;
        let mut data_len = data_len.to_le_bytes().to_vec();

        data_len.append(&mut serialised_data);
        data_len

        // vec![0,0,0,4]
    }
}
