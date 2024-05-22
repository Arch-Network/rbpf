use std::mem::size_of;

use solana_rbpf::{
    aligned_memory::{AlignedMemory, Pod},
    ebpf::{HOST_ALIGN, MM_INPUT_START},
    memory_region::{MemoryRegion, MemoryState},
};

use crate::{consts::*, types::*, error::*};

/// TODO:
/// Define a better format for serde,
/// better to do it after the inital prototype is ready so better understanding of code is available

struct Serializer {
    pub buffer: AlignedMemory<HOST_ALIGN>,
    regions: Vec<MemoryRegion>,
    vaddr: u64,
    region_start: usize,
    aligned: bool,
    copy_account_data: bool,
}

impl Serializer {
    fn new(size: usize, start_addr: u64, aligned: bool, copy_account_data: bool) -> Serializer {
        Serializer {
            buffer: AlignedMemory::with_capacity(size),
            regions: Vec::new(),
            region_start: 0,
            vaddr: start_addr,
            aligned,
            copy_account_data,
        }
    }

    fn fill_write(&mut self, num: usize, value: u8) -> std::io::Result<()> {
        self.buffer.fill_write(num, value)
    }

    pub fn write<T: Pod>(&mut self, value: T) -> u64 {
        self.debug_assert_alignment::<T>();
        let vaddr = self
            .vaddr
            .saturating_add(self.buffer.len() as u64)
            .saturating_sub(self.region_start as u64);
        // Safety:
        // in serialize_parameters_(aligned|unaligned) first we compute the
        // required size then we write into the newly allocated buffer. There's
        // no need to check bounds at every write.
        //
        // AlignedMemory::write_unchecked _does_ debug_assert!() that the capacity
        // is enough, so in the unlikely case we introduce a bug in the size
        // computation, tests will abort.
        unsafe {
            self.buffer.write_unchecked(value);
        }

        vaddr
    }

    fn write_all(&mut self, value: &[u8]) -> u64 {
        let vaddr = self
            .vaddr
            .saturating_add(self.buffer.len() as u64)
            .saturating_sub(self.region_start as u64);
        // Safety:
        // see write() - the buffer is guaranteed to be large enough
        unsafe {
            self.buffer.write_all_unchecked(value);
        }

        vaddr
    }

    fn write_account(
        &mut self,
        account: &mut BorrowedAccount<'_>,
    ) -> Result<u64, InstructionError> {
        let vm_data_addr = if self.copy_account_data {
            let vm_data_addr = self.vaddr.saturating_add(self.buffer.len() as u64);
            self.write_all(account.get_data());
            vm_data_addr
        } else {
            self.push_region(true);
            let vaddr = self.vaddr;
            self.push_account_data_region(account)?;
            vaddr
        };

        if self.aligned {
            let align_offset =
                (account.get_data().len() as *const u8).align_offset(BPF_ALIGN_OF_U128);
            if self.copy_account_data {
                self.fill_write(MAX_PERMITTED_DATA_INCREASE + align_offset, 0)
                    .map_err(|_| InstructionError::InvalidArgument)?;
            } else {
                // The deserialization code is going to align the vm_addr to
                // BPF_ALIGN_OF_U128. Always add one BPF_ALIGN_OF_U128 worth of
                // padding and shift the start of the next region, so that once
                // vm_addr is aligned, the corresponding host_addr is aligned
                // too.
                self.fill_write(MAX_PERMITTED_DATA_INCREASE + BPF_ALIGN_OF_U128, 0)
                    .map_err(|_| InstructionError::InvalidArgument)?;
                self.region_start += BPF_ALIGN_OF_U128.saturating_sub(align_offset);
                // put the realloc padding in its own region
                self.push_region(account.can_data_be_changed().is_ok());
            }
        }

        Ok(vm_data_addr)
    }

    // fn push_account_data_region(
    //     &mut self,
    //     account: &mut BorrowedAccount<'_>,
    // ) -> Result<(), InstructionError> {
    //     if !account.get_data().is_empty() {
    //         let region = match account_data_region_memory_state(account) {
    //             MemoryState::Readable => MemoryRegion::new_readonly(account.get_data(), self.vaddr),
    //             MemoryState::Writable => {
    //                 MemoryRegion::new_writable(account.get_data_mut()?, self.vaddr)
    //             }
    //             MemoryState::Cow(index_in_transaction) => {
    //                 MemoryRegion::new_cow(account.get_data(), self.vaddr, index_in_transaction)
    //             }
    //         };
    //         self.vaddr += region.len;
    //         self.regions.push(region);
    //     }

    //     Ok(())
    // }

    fn push_region(&mut self, writable: bool) {
        let range = self.region_start..self.buffer.len();
        let region = if writable {
            MemoryRegion::new_writable(
                self.buffer.as_slice_mut().get_mut(range.clone()).unwrap(),
                self.vaddr,
            )
        } else {
            MemoryRegion::new_readonly(
                self.buffer.as_slice().get(range.clone()).unwrap(),
                self.vaddr,
            )
        };
        self.regions.push(region);
        self.region_start = range.end;
        self.vaddr += range.len() as u64;
    }

    fn finish(mut self) -> (AlignedMemory<HOST_ALIGN>, Vec<MemoryRegion>) {
        self.push_region(true);
        debug_assert_eq!(self.region_start, self.buffer.len());
        (self.buffer, self.regions)
    }

    fn debug_assert_alignment<T>(&self) {
        debug_assert!(
            !self.aligned
                || self
                    .buffer
                    .as_slice()
                    .as_ptr_range()
                    .end
                    .align_offset(std::mem::align_of::<T>())
                    == 0
        );
    }
}

fn serialize_parameters_aligned(
    accounts: Vec<SerializeAccount>,
    instruction_data: &[u8],
    program_id: &Pubkey,
    copy_account_data: bool,
) -> Result<
    (
        AlignedMemory<HOST_ALIGN>,
        Vec<MemoryRegion>,
        Vec<SerializedAccountMetadata>,
    ),
    InstructionError,
> {
    let mut accounts_metadata = Vec::with_capacity(accounts.len());
    // Calculate size in order to alloc once
    let mut size = size_of::<u64>();
    for account in &accounts {
        size += 1; // dup
        match account {
            SerializeAccount::Duplicate(_) => size += 7, // padding to 64-bit aligned
            SerializeAccount::Account(_, account) => {
                let data_len = account.get_data().len();
                size += size_of::<u8>() // is_signer
                + size_of::<u8>() // is_writable
                + size_of::<u8>() // executable
                + size_of::<u32>() // original_data_len
                + size_of::<Pubkey>()  // key
                + size_of::<Pubkey>() // owner
                + size_of::<u64>()  // lamports
                + size_of::<u64>()  // data len
                + MAX_PERMITTED_DATA_INCREASE
                + size_of::<u64>(); // rent epoch
                if copy_account_data {
                    size += data_len + (data_len as *const u8).align_offset(BPF_ALIGN_OF_U128);
                } else {
                    size += BPF_ALIGN_OF_U128;
                }
            }
        }
    }
    size += size_of::<u64>() // data len
    + instruction_data.len()
    + size_of::<Pubkey>(); // program id;

    let mut s = Serializer::new(size, MM_INPUT_START, true, copy_account_data);

    // Serialize into the buffer
    s.write::<u64>((accounts.len() as u64).to_le());
    for account in accounts {
        match account {
            SerializeAccount::Account(_, mut borrowed_account) => {
                s.write::<u8>(NON_DUP_MARKER);
                s.write::<u8>(borrowed_account.is_signer() as u8);
                s.write::<u8>(borrowed_account.is_writable() as u8);
                s.write::<u8>(borrowed_account.is_executable() as u8);
                s.write_all(&[0u8, 0, 0, 0]);
                let vm_key_addr = s.write_all(borrowed_account.get_key().as_ref());
                let vm_owner_addr = s.write_all(borrowed_account.get_owner().as_ref());
                let vm_lamports_addr = s.write::<u64>(borrowed_account.get_lamports().to_le());
                s.write::<u64>((borrowed_account.get_data().len() as u64).to_le());
                let vm_data_addr = s.write_account(&mut borrowed_account)?;
                s.write::<u64>((borrowed_account.get_rent_epoch()).to_le());
                accounts_metadata.push(SerializedAccountMetadata {
                    original_data_len: borrowed_account.get_data().len(),
                    vm_key_addr,
                    vm_owner_addr,
                    vm_lamports_addr,
                    vm_data_addr,
                });
            }
            SerializeAccount::Duplicate(position) => {
                accounts_metadata.push(accounts_metadata.get(position as usize).unwrap().clone());
                s.write::<u8>(position as u8);
                s.write_all(&[0u8, 0, 0, 0, 0, 0, 0]);
            }
        };
    }
    s.write::<u64>((instruction_data.len() as u64).to_le());
    s.write_all(instruction_data);
    s.write_all(program_id.as_ref());

    let (mem, regions) = s.finish();
    Ok((mem, regions, accounts_metadata))
}
