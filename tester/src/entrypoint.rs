use std::{alloc::Layout, collections::HashMap, mem::size_of, ptr::null_mut};
use borsh::from_slice;
use crate::types::*;
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

pub fn deserialize<'a>(input : *mut u8) -> (Pubkey, Vec<UtxoInfo>, Vec<u8>){
    let size = unsafe { *(input as *mut u32)};
    let data_slice = unsafe { std::slice::from_raw_parts_mut(input.add(4), size as usize)};

    let (instruction, authorities,data): (Instruction,HashMap<String, Vec<u8>>,HashMap<String, Vec<u8>> ) = from_slice(&data_slice).expect("unable to deserialise input to entrypoint function");

            let program_id: Pubkey = instruction.program_id;

            let utxos = instruction
                .utxos
                .iter()
                .map(|utxo| {
                    use std::cell::RefCell;
                    UtxoInfo {
                        txid: utxo.txid.clone(),
                        vout: utxo.vout,
                        authority: RefCell::new(Pubkey(
                            authorities
                                .get(&utxo.id())
                                .expect("this utxo does not exist in auth")
                                .to_vec(),
                        )),
                        data: RefCell::new(
                            data.get(&utxo.id())
                                .expect("this utxo does not exist in data")
                                .to_vec(),
                        ),
                    }
                })
                .collect::<Vec<UtxoInfo>>();
            let instruction_data: Vec<u8> = instruction.data;

            (program_id, utxos, instruction_data)

}