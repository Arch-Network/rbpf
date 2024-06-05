// // use std::{alloc::Layout, collections::HashMap, mem::size_of, ptr::null_mut};
// // use borsh::from_slice;
// // extern crate alloc;
// // use alloc::vec::Vec;
// // use crate::types::*;
// // /// Start address of the memory region used for program heap.
// // pub const HEAP_START_ADDRESS: u64 = 0x300000000;
// // /// Length of the heap memory region used for program heap.
// // pub const HEAP_LENGTH: usize = 32 * 1024;

// // /// The bump allocator used as the default rust heap when running programs.
// // pub struct BumpAllocator {
// //     pub start: usize,
// //     pub len: usize,
// // }
// // /// Integer arithmetic in this global allocator implementation is safe when
// // /// operating on the prescribed `HEAP_START_ADDRESS` and `HEAP_LENGTH`. Any
// // /// other use may overflow and is thus unsupported and at one's own risk.
// // #[allow(clippy::arithmetic_side_effects)]
// // unsafe impl std::alloc::GlobalAlloc for BumpAllocator {
// //     #[inline]
// //     unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
// //         let pos_ptr = self.start as *mut usize;

// //         let mut pos = *pos_ptr;
// //         if pos == 0 {
// //             // First time, set starting position
// //             pos = self.start + self.len;
// //         }
// //         pos = pos.saturating_sub(layout.size());
// //         pos &= !(layout.align().wrapping_sub(1));
// //         if pos < self.start + size_of::<*mut u8>() {
// //             return null_mut();
// //         }
// //         *pos_ptr = pos;
// //         pos as *mut u8
// //     }
// //     #[inline]
// //     unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
// //         // I'm a bump allocator, I don't free
// //     }
// // }

// // pub fn deserialize(input : *mut u8) -> (String, u32) {
// //     let size = unsafe { *(input as * mut u32) };
// //     let data_slice = unsafe { std::slice::from_raw_parts_mut(input.add(4), size as usize)};
// //     let string: String = from_slice(&data_slice).expect("unable to deserialise input to entrypoint function");
// //     (string,size + 1)
// // }

// // #[macro_export]
// // macro_rules! entrypoint {
// //     ($process_instruction:ident) => {
// //         /// # Safety
// //         #[no_mangle]
// //         pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
// //             let (string, size) =
// //                 unsafe { $crate::dentrypoint::deserialize(input) };
// //             match $process_instruction(string, size) {
// //                 Ok(_) => {
// //                     unsafe {*(input as *mut u32) = size + 1;}
// //                     return 0;
// //                 }
// //                 Err(e) => {
// //                     return 1;
// //                 }
// //             }

// //         }
// //         $crate::custom_heap_default!();
// //         // $crate::custom_panic_default!();

// //     };
// // }

// // #[macro_export]
// // // macro_rules! custom_panic_default {
// // //     () => {
// // //         #[no_mangle]
// // //         fn custom_panic(info: &core::panic::PanicInfo<'_>) {
// // //             // Full panic reporting
// // //             msg!("{}",info);
// // //         }
// // //     };
// // // }

// // // #[macro_export]
// // // macro_rules! msg {
// // //     ($msg:expr) => {
// // //         $crate::entrypoint::sol_log($msg)
// // //     };
// // //     ($($arg:tt)*) => ($crate::dentrypoint::sol_log(&format!($($arg)*)));
// // // }

// // /// Print a string to the log.
// // // #[inline]
// // // pub fn sol_log(message: &str) {
// // //     unsafe {
// // //         crate::dentrypoint::sol_log_(message.as_ptr(), message.len() as u64);
// // //     }
// // // }
// // #[macro_export]
// // macro_rules! custom_heap_default {
// //     () => {
// //         #[global_allocator]
// //         static A: $crate::dentrypoint::BumpAllocator = $crate::dentrypoint::BumpAllocator {
// //             start: $crate::dentrypoint::HEAP_START_ADDRESS as usize,
// //             len: $crate::dentrypoint::HEAP_LENGTH,
// //         };
// //     };
// // }

// // // macro_rules! define_syscall {
// // // 	(fn $name:ident($($arg:ident: $typ:ty),*) -> $ret:ty) => {
// // // 		extern "C" {
// // // 			pub fn $name($($arg: $typ),*) -> $ret;
// // // 		}
// // // 	};
// // // 	(fn $name:ident($($arg:ident: $typ:ty),*)) => {
// // // 		define_syscall!(fn $name($($arg: $typ),*) -> ());
// // // 	}
// // // }

// // // define_syscall!(fn sol_log_(message: *const u8, len: u64));


// // // mod Test {
// // //     use super::*;

// // //     #[test]
// // //     fn test_entrypoint() {
// // //         println!("Hello");
// // //         // #[global_allocator]
// // //         // static A: BumpAllocator = BumpAllocator {
// // //         //     start: HEAP_START_ADDRESS as usize,
// // //         //     len: HEAP_LENGTH,
// // //         // };

// // //         entrypoint!(process_instruction);

// // //         }
// // //     pub fn process_instruction(size : u32) -> Result<(),String> {

// // //         // let txin = TxIn {
// // //         //      txid: String::from("abcdef"),
// // //         //      vout:1,
// // //         //      script_sig: [12u8;32].to_vec(),
// // //         //      sequence: 5,
// // //         //      witness: vec![[12u8;32].to_vec(), [22u8;34].to_vec()]
// // //         // };
    
// // //         // let txout = TxOut {
// // //         //     amount: 10240,
// // //         //     script_pubkey: [122u8;64].to_vec(),
// // //         // };
    
// // //         // return Ok(Transaction {
// // //         //     version: 1,
// // //         //     input: vec![txin],
// // //         //     output: vec![txout],
// // //         //     lock_time: 15,
// // //         // })
// // //         Ok(())
// // //     }

// // //     pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
// // //         let size =
// // //             unsafe { deserialize(input) };
// // //         match process_instruction(size) {
// // //             Ok(tx_hex) => {
// // //                 unsafe {*(input as *mut u32) = size + 1  as u32;}
// // //                 return 0;
// // //             }
// // //             Err(e) => {
// // //                 return 1;
// // //             }
// // //         }
// // // }
// // // }





// //* End Of Part 1 */

// use std::{alloc::Layout, collections::HashMap, mem::size_of, ptr::{null_mut, slice_from_raw_parts, slice_from_raw_parts_mut}, slice::from_raw_parts};
// // use borsh::from_slice;
// extern crate alloc;
// use alloc::vec::Vec;
// use crate::types::*;
// /// Start address of the memory region used for program heap.
// pub const HEAP_START_ADDRESS: u64 = 0x300000000;
// /// Length of the heap memory region used for program heap.
// pub const HEAP_LENGTH: usize = 32 * 1024;

// /// The bump allocator used as the default rust heap when running programs.
// pub struct BumpAllocator {
//     pub start: usize,
//     pub len: usize,
// }

// #[allow(clippy::arithmetic_side_effects)]
// unsafe impl std::alloc::GlobalAlloc for BumpAllocator {
//     #[inline]
//     unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
//         let pos_ptr = self.start as *mut usize;

//         let mut pos = *pos_ptr;
//         if pos == 0 {
//             // First time, set starting position
//             pos = self.start + self.len;
//         }
//         pos = pos.saturating_sub(layout.size());
//         pos &= !(layout.align().wrapping_sub(1));
//         if pos < self.start + size_of::<*mut u8>() {
//             return null_mut();
//         }
//         *pos_ptr = pos;
//         pos as *mut u8
//     }
//     #[inline]
//     unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
//         // I'm a bump allocator, I don't free
//     }
// }

// // function that doesn't use borch

// pub unsafe fn deserialize(input : *mut u8) -> Vec<u8> {
//     // let size = unsafe { *(input as * mut u32) };
//     let vec_slice = unsafe { slice_from_raw_parts_mut(input, 9).as_ref().expect("string conversion error in deserlisation") };
//     vec_slice.to_vec()
//     // let instruction_data = { from_raw_parts(input, 9) };
//     // return instruction_data.to_vec()
// }

// // using borsh

// // pub fn deserialize(input : *mut u8) -> (String, u32) {
// //     let size = unsafe { *(input as * mut u32) };
// //     let data_slice = unsafe { std::slice::from_raw_parts_mut(input.add(4), size as usize)};
// //     let string = borsh::from_slice::<String>(&data_slice).expect("Invalid UTF-8 sequence in deserliastion entrypoint");
// //     (string,size)
// // }

// #[macro_export]
// macro_rules! entrypoint {
//     ($process_instruction:ident) => {
//         /// # Safety
//         #[no_mangle]
//         pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
//             use std::collections::HashMap;
//             let ins =
//                 unsafe { $crate::dentrypoint::deserialize(input) };
//             match $process_instruction(ins) {
//                 Ok(tx_hex) => {
//                     let mut new_authorities: HashMap<String, Vec<u8>> = HashMap::new();
//                     let mut new_data: HashMap<String, Vec<u8>> = HashMap::new();
//                     let mut serialised_output = borsh::to_vec(&(new_authorities, new_data, tx_hex)).unwrap();
//                     let output_len = serialised_output.len();
//                     println!("output length is {}", output_len);
//                     unsafe {*(input as *mut u32) = output_len as u32;}

//                     unsafe {
//                         std::ptr::copy_nonoverlapping(serialised_output.as_mut_ptr(),input.add(4),output_len);
//                     }
//                     return 0;
//                 }
//                 Err(e) => {
//                 return 1;
//                 }
//             }

//         }
//         $crate::custom_heap_default!();
//         // $crate::custom_panic_default!();

//     };
// }

// #[macro_export]
// // macro_rules! custom_panic_default {
// //     () => {
// //         #[no_mangle]
// //         fn custom_panic(info: &core::panic::PanicInfo<'_>) {
// //             // Full panic reporting
// //             msg!("{}",info);
// //         }
// //     };
// // }

// // #[macro_export]
// // macro_rules! msg {
// //     ($msg:expr) => {
// //         $crate::entrypoint::sol_log($msg)
// //     };
// //     ($($arg:tt)*) => ($crate::dentrypoint::sol_log(&format!($($arg)*)));
// // }

// /// Print a string to the log.
// // #[inline]
// // pub fn sol_log(message: &str) {
// //     unsafe {
// //         crate::dentrypoint::sol_log_(message.as_ptr(), message.len() as u64);
// //     }
// // }

// #[macro_export]
// macro_rules! custom_heap_default {
//     () => {
//         #[global_allocator]
//         static A: $crate::dentrypoint::BumpAllocator = $crate::dentrypoint::BumpAllocator {
//             start: $crate::dentrypoint::HEAP_START_ADDRESS as usize,
//             len: $crate::dentrypoint::HEAP_LENGTH,
//         };
//     };
// }

// // macro_rules! define_syscall {
// // 	(fn $name:ident($($arg:ident: $typ:ty),*) -> $ret:ty) => {
// // 		extern "C" {
// // 			pub fn $name($($arg: $typ),*) -> $ret;
// // 		}
// // 	};
// // 	(fn $name:ident($($arg:ident: $typ:ty),*)) => {
// // 		define_syscall!(fn $name($($arg: $typ),*) -> ());
// // 	}
// // }

// // define_syscall!(fn sol_log_(message: *const u8, len: u64));


// // mod Test {
// //     use super::*;

// //     #[test]
// //     fn test_entrypoint() {
// //         println!("Hello");
// //         // #[global_allocator]
// //         // static A: BumpAllocator = BumpAllocator {
// //         //     start: HEAP_START_ADDRESS as usize,
// //         //     len: HEAP_LENGTH,
// //         // };

// //         entrypoint!(process_instruction);

// //         }
// //     pub fn process_instruction(size : u32) -> Result<(),String> {

// //         // let txin = TxIn {
// //         //      txid: String::from("abcdef"),
// //         //      vout:1,
// //         //      script_sig: [12u8;32].to_vec(),
// //         //      sequence: 5,
// //         //      witness: vec![[12u8;32].to_vec(), [22u8;34].to_vec()]
// //         // };
    
// //         // let txout = TxOut {
// //         //     amount: 10240,
// //         //     script_pubkey: [122u8;64].to_vec(),
// //         // };
    
// //         // return Ok(Transaction {
// //         //     version: 1,
// //         //     input: vec![txin],
// //         //     output: vec![txout],
// //         //     lock_time: 15,
// //         // })
// //         Ok(())
// //     }

// //     pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
// //         let size =
// //             unsafe { deserialize(input) };
// //         match process_instruction(size) {
// //             Ok(tx_hex) => {
// //                 unsafe {*(input as *mut u32) = size + 1  as u32;}
// //                 return 0;
// //             }
// //             Err(e) => {
// //                 return 1;
// //             }
// //         }
// // }
// // // }

// //* END of part 2 *//




// // extern crate alloc;

// //  pub type Epoch = u64;
// // /// Account information
// // #[derive(Clone,Debug)]
// // pub struct AccountInfo<'a> {
// //     /// Public key of the account
// //     pub key: &'a Pubkey,
// //     /// The lamports in the account.  Modifiable by programs.
// //     pub lamports: Rc<RefCell<&'a mut u64>>,
// //     /// The data held in this account.  Modifiable by programs.
// //     pub data: Rc<RefCell<&'a mut [u8]>>,
// //     /// Program that owns this account
// //     pub owner: &'a Pubkey,
// //     /// The epoch at which this account will next owe rent
// //     pub rent_epoch: Epoch,
// //     /// Was the transaction signed by this account's public key?
// //     pub is_signer: bool,
// //     /// Is the account writable?
// //     pub is_writable: bool,
// //     /// This account's data contains a loaded program (and is now read-only)
// //     pub executable: bool,
// // }

// // Reasons the program may fail
// // #[derive(Clone, Debug, Deserialize, Eq, Error, PartialEq, Serialize)]
// // pub enum ProgramError {
// //     /// Allows on-chain programs to implement program-specific error types and see them returned
// //     /// by the Solana runtime. A program-specific error may be any type that is represented as
// //     /// or serialized to a u32 integer.
// //     #[error("Custom program error: {0:#x}")]
// //     Custom(u32),
// //     #[error("The arguments provided to a program instruction were invalid")]
// //     InvalidArgument,
// //     #[error("An instruction's data contents was invalid")]
// //     InvalidInstructionData,
// //     #[error("An account's data contents was invalid")]
// //     InvalidAccountData,
// //     #[error("An account's data was too small")]
// //     AccountDataTooSmall,
// //     #[error("An account's balance was too small to complete the instruction")]
// //     InsufficientFunds,
// //     #[error("The account did not have the expected program id")]
// //     IncorrectProgramId,
// //     #[error("A signature was required but not found")]
// //     MissingRequiredSignature,
// //     #[error("An initialize instruction was sent to an account that has already been initialized")]
// //     AccountAlreadyInitialized,
// //     #[error("An attempt to operate on an account that hasn't been initialized")]
// //     UninitializedAccount,
// //     #[error("The instruction expected additional account keys")]
// //     NotEnoughAccountKeys,
// //     #[error("Failed to borrow a reference to account data, already borrowed")]
// //     AccountBorrowFailed,
// //     #[error("Length of the seed is too long for address generation")]
// //     MaxSeedLengthExceeded,
// //     #[error("Provided seeds do not result in a valid address")]
// //     InvalidSeeds,
// //     #[error("IO Error: {0}")]
// //     BorshIoError(String),
// //     #[error("An account does not have enough lamports to be rent-exempt")]
// //     AccountNotRentExempt,
// //     #[error("Unsupported sysvar")]
// //     UnsupportedSysvar,
// //     #[error("Provided owner is not allowed")]
// //     IllegalOwner,
// //     #[error("Accounts data allocations exceeded the maximum allowed per transaction")]
// //     MaxAccountsDataAllocationsExceeded,
// //     #[error("Account data reallocation was invalid")]
// //     InvalidRealloc,
// //     #[error("Instruction trace length exceeded the maximum allowed per transaction")]
// //     MaxInstructionTraceLengthExceeded,
// //     #[error("Builtin programs must consume compute units")]
// //     BuiltinProgramsMustConsumeComputeUnits,
// //     #[error("Invalid account owner")]
// //     InvalidAccountOwner,
// //     #[error("Program arithmetic overflowed")]
// //     ArithmeticOverflow,
// //     #[error("Account is immutable")]
// //     Immutable,
// //     #[error("Incorrect authority provided")]
// //     IncorrectAuthority,
// // }






// // START OF PART Solana

// // #[derive(Debug)]
// // pub struct Pubkey(pub(crate) [u8; 32]);

// // use {
// //     alloc::vec::Vec,
// //     std::{
// //         alloc::Layout,
// //         cell::RefCell,
// //         mem::size_of,
// //         ptr::null_mut,
// //         rc::Rc,
// //         result::Result as ResultGeneric,
// //         slice::{from_raw_parts, from_raw_parts_mut},
// //     },
// // };


// // /// Programs indicate success with a return value of 0
// // pub const SUCCESS: u64 = 0;

// // /// Start address of the memory region used for program heap.
// // pub const HEAP_START_ADDRESS: u64 = 0x300000000;
// // /// Length of the heap memory region used for program heap.
// // pub const HEAP_LENGTH: usize = 32 * 1024;

// // /// Value used to indicate that a serialized account is not a duplicate
// // pub const NON_DUP_MARKER: u8 = u8::MAX;

// // #[macro_export]
// // macro_rules! entrypoint {
// //     ($process_instruction:ident) => {
// //         /// # Safety
// //         #[no_mangle]
// //         pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
// //             let instruction_data =
// //                 unsafe { $crate::dentrypoint::deserialize(input) };
// //             match $process_instruction(instruction_data) {
// //                 Ok(()) => $crate::dentrypoint::SUCCESS,
// //                 Err(error) => 1,
// //             }
// //         }
// //         // $crate::custom_heap_default!();
// //         // $crate::custom_panic_default!();
// //     };
// // }

// // /// Define the default global allocator.
// // ///
// // /// The default global allocator is enabled only if the calling crate has not
// // /// disabled it using [Cargo features] as described below. It is only defined
// // /// for [BPF] targets.
// // ///
// // /// [Cargo features]: https://doc.rust-lang.org/cargo/reference/features.html
// // /// [BPF]: https://solana.com/docs/programs/faq#berkeley-packet-filter-bpf
// // ///
// // /// # Cargo features
// // ///
// // /// A crate that calls this macro can provide its own custom heap
// // /// implementation, or allow others to provide their own custom heap
// // /// implementation, by adding a `custom-heap` feature to its `Cargo.toml`. After
// // /// enabling the feature, one may define their own [global allocator] in the
// // /// standard way.
// // ///
// // /// [global allocator]: https://doc.rust-lang.org/stable/std/alloc/trait.GlobalAlloc.html
// // ///
// // #[macro_export]
// // macro_rules! custom_heap_default {
// //     () => {
// //         #[cfg(all(not(feature = "custom-heap"), target_os = "solana"))]
// //         #[global_allocator]
// //         static A: $crate::dentrypoint::BumpAllocator = $crate::dentrypoint::BumpAllocator {
// //             start: $crate::dentrypoint::HEAP_START_ADDRESS as usize,
// //             len: $crate::dentrypoint::HEAP_LENGTH,
// //         };
// //     };
// // }

// // pub struct BumpAllocator {
// //     pub start: usize,
// //     pub len: usize,
// // }
// // /// Integer arithmetic in this global allocator implementation is safe when
// // /// operating on the prescribed `HEAP_START_ADDRESS` and `HEAP_LENGTH`. Any
// // /// other use may overflow and is thus unsupported and at one's own risk.
// // #[allow(clippy::arithmetic_side_effects)]
// // unsafe impl std::alloc::GlobalAlloc for BumpAllocator {
// //     #[inline]
// //     unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
// //         let pos_ptr = self.start as *mut usize;

// //         let mut pos = *pos_ptr;
// //         if pos == 0 {
// //             // First time, set starting position
// //             pos = self.start + self.len;
// //         }
// //         pos = pos.saturating_sub(layout.size());
// //         pos &= !(layout.align().wrapping_sub(1));
// //         if pos < self.start + size_of::<*mut u8>() {
// //             return null_mut();
// //         }
// //         *pos_ptr = pos;
// //         pos as *mut u8
// //     }
// //     #[inline]
// //     unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
// //         // I'm a bump allocator, I don't free
// //     }
// // }

// // /// Maximum number of bytes a program may add to an account during a single realloc
// // pub const MAX_PERMITTED_DATA_INCREASE: usize = 1_024 * 10;

// // /// `assert_eq(std::mem::align_of::<u128>(), 8)` is true for BPF but not for some host machines
// // pub const BPF_ALIGN_OF_U128: usize = 8;

// // /// Deserialize the input arguments
// // ///
// // /// The integer arithmetic in this method is safe when called on a buffer that was
// // /// serialized by runtime. Use with buffers serialized otherwise is unsupported and
// // /// done at one's own risk.
// // ///
// // /// # Safety
// // #[allow(clippy::arithmetic_side_effects)]
// // #[allow(clippy::type_complexity)]
// // pub unsafe fn deserialize<'a>(input: *mut u8) -> 
// //     Vec<u8> {
// //     let mut offset: usize = 0;

// //     // Number of accounts present

// //     // #[allow(clippy::cast_ptr_alignment)]
// //     // let num_accounts = *(input.add(offset) as *const u64) as usize;
// //     // offset += size_of::<u64>();

// //     // // Account Infos

// //     // let mut accounts = Vec::with_capacity(num_accounts);
// //     // for _ in 0..num_accounts {
// //     //     let dup_info = *(input.add(offset) as *const u8);
// //     //     offset += size_of::<u8>();
// //     //     if dup_info == NON_DUP_MARKER {
// //     //         #[allow(clippy::cast_ptr_alignment)]
// //     //         let is_signer = *(input.add(offset) as *const u8) != 0;
// //     //         offset += size_of::<u8>();

// //     //         #[allow(clippy::cast_ptr_alignment)]
// //     //         let is_writable = *(input.add(offset) as *const u8) != 0;
// //     //         offset += size_of::<u8>();

// //     //         #[allow(clippy::cast_ptr_alignment)]
// //     //         let executable = *(input.add(offset) as *const u8) != 0;
// //     //         offset += size_of::<u8>();

// //     //         // The original data length is stored here because these 4 bytes were
// //     //         // originally only used for padding and served as a good location to
// //     //         // track the original size of the account data in a compatible way.
// //     //         let original_data_len_offset = offset;
// //     //         offset += size_of::<u32>();

// //     //         let key: &Pubkey = &*(input.add(offset) as *const Pubkey);
// //     //         offset += size_of::<Pubkey>();

// //     //         let owner: &Pubkey = &*(input.add(offset) as *const Pubkey);
// //     //         offset += size_of::<Pubkey>();

// //     //         #[allow(clippy::cast_ptr_alignment)]
// //     //         let lamports = Rc::new(RefCell::new(&mut *(input.add(offset) as *mut u64)));
// //     //         offset += size_of::<u64>();

// //     //         #[allow(clippy::cast_ptr_alignment)]
// //     //         let data_len = *(input.add(offset) as *const u64) as usize;
// //     //         offset += size_of::<u64>();

// //     //         // Store the original data length for detecting invalid reallocations and
// //     //         // requires that MAX_PERMITTED_DATA_LENGTH fits in a u32
// //     //         *(input.add(original_data_len_offset) as *mut u32) = data_len as u32;

// //     //         let data = Rc::new(RefCell::new({
// //     //             from_raw_parts_mut(input.add(offset), data_len)
// //     //         }));
// //     //         offset += data_len + MAX_PERMITTED_DATA_INCREASE;
// //     //         offset += (offset as *const u8).align_offset(BPF_ALIGN_OF_U128); // padding

// //     //         #[allow(clippy::cast_ptr_alignment)]
// //     //         let rent_epoch = *(input.add(offset) as *const u64);
// //     //         offset += size_of::<u64>();

// //     //         accounts.push(AccountInfo {
// //     //             key,
// //     //             is_signer,
// //     //             is_writable,
// //     //             lamports,
// //     //             data,
// //     //             owner,
// //     //             executable,
// //     //             rent_epoch,
// //     //         });
// //     //     } else {
// //     //         offset += 7; // padding

// //     //         // Duplicate account, clone the original
// //     //         accounts.push(accounts[dup_info as usize].clone());
// //     //     }
// //     // }

// //     // // Instruction data

// //     // #[allow(clippy::cast_ptr_alignment)]
// //     // let instruction_data_len = *(input.add(offset) as *const u64) as usize;
// //     // offset += size_of::<u64>();

// //     let instruction_data = { from_raw_parts(input.add(offset), 9) };
// //     offset += 9;

// //     // Program Id

// //     // let program_id: &Pubkey = &*(input.add(offset) as *const Pubkey);

// //     instruction_data.to_vec()
// //     // (program_id, accounts, instruction_data)
// // }
