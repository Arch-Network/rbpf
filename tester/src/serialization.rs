use std::mem::{self, size_of};

use byteorder::{ByteOrder, LittleEndian};
use core_types::{entrypoint::{BPF_ALIGN_OF_U128, MAX_PERMITTED_DATA_INCREASE, MAX_PERMITTED_DATA_LENGTH, NON_DUP_MARKER}, types::Pubkey};
use solana_rbpf::{aligned_memory::{AlignedMemory, Pod}, ebpf::{HOST_ALIGN,MM_INPUT_START}, memory_region::MemoryRegion,};

enum SerializeUtxo<'a> {
    Account(IndexOfUtxo, BorrowedUtxo<'a>),
    Duplicate(IndexOfUtxo),
}

struct Serializer {
    pub buffer: AlignedMemory<HOST_ALIGN>,
    regions: Vec<MemoryRegion>,
    vaddr: u64,
    region_start: usize,
}

impl Serializer {
    fn new(size: usize, start_addr: u64) -> Serializer {
        Serializer {
            buffer: AlignedMemory::with_capacity(size),
            regions: Vec::new(),
            region_start: 0,
            vaddr: start_addr,
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

    fn finish(mut self) -> AlignedMemory<HOST_ALIGN> {
        self.buffer
    }


    fn debug_assert_alignment<T>(&self) {
        debug_assert!(
          self
            .buffer
            .as_slice()
            .as_ptr_range()
            .end
            .align_offset(mem::align_of::<T>())
            == 0
        );
    }

    fn write_utxo(
        &mut self,
        utxo: &mut BorrowedUtxo<'_>,
    ) -> Result<u64, InstructionError> {
        let vm_data_addr = self.vaddr.saturating_add(self.buffer.len() as u64);
        self.write_all(utxo.get_data());


        let align_offset =
            (utxo.get_data().len() as *const u8).align_offset(BPF_ALIGN_OF_U128);
        
        self.fill_write(MAX_PERMITTED_DATA_INCREASE + align_offset, 0)
            .map_err(|_| InstructionError::InvalidArgument)?;

        Ok(vm_data_addr)
    }
}


use crate::{errors::InstructionError, processor::{BorrowedUtxo, IndexOfUtxo, InstructionContext, SerializedUtxoMetadata, TransactionContext}};

/// Information about Data SerDe process
/// We have to keep in mind that every thing should be aligned
/// DATA :
/// Total Utxos : [u64]
/// 
/// DUPLICATE ENTRY:
/// INDEX of Duplicated Utxo <u8> followed by 7 bytes of padding
/// 
/// NON DUPLICATE ENTRY:
/// NON_DUP_MARKER [u8] followed by 3 bytes of padding 
/// vout of BTC TXN [u32]
/// 
/// BTC TXNID [u8;32]
/// Data of UTXO -> arbitrary size
/// len of data -> u32/u64
/// Authority -> [u8;32]
/// 
/// Instruction_data_len : u64
/// Instruction data : [u8]
/// Pubkey of program -> [u8;32]


pub fn serialize_parameters(
    transaction_context: &TransactionContext,
    instruction_context: &InstructionContext,
) -> Result<
    (
        AlignedMemory<HOST_ALIGN>,
        Vec<SerializedUtxoMetadata>,
    ),
    InstructionError,
> {
    let num_ix_accounts = instruction_context.get_number_of_instruction_utxos();
    if num_ix_accounts > NON_DUP_MARKER as IndexOfUtxo {
        return Err(InstructionError::MaxAccountsExceeded);
    }

    let program_id = *instruction_context.get_last_program_key();
    let instruction_data = instruction_context.get_instruction_data();

    let utxos = (0..instruction_context.get_number_of_instruction_utxos())
        .map(|instruction_account_index| {
            if let Some(index) = instruction_context
                .is_instruction_utxo_duplicate(instruction_account_index)
                .unwrap()
            {
                SerializeUtxo::Duplicate(index)
            } else {
                let account = instruction_context
                    .try_borrow_instruction_utxo(transaction_context, instruction_account_index)
                    .unwrap();
                SerializeUtxo::Account(instruction_account_index, account)
            }
        })
        .collect::<Vec<_>>();
        
        let mut utxos_metadata = Vec::with_capacity(utxos.len());
        // Calculate size in order to alloc once
        let mut size = size_of::<u64>(); // total utxos

        for utxo in &utxos {
            size += 1; // dup

            match utxo {
                SerializeUtxo::Duplicate(_) => size += 7, // padding to 64-bit aligned
                SerializeUtxo::Account(_, utxo) => {
                    let data_len = utxo.data_len();

                    size += 3 // padding for 3 bytes
                    + size_of::<u32>() // vout
                    + size_of::<[u8;32]>() // Txid for btc txn
                    + size_of::<Pubkey>() // Authority
                    + size_of::<u64>() // original data len
                    + size_of::<u64>() // current data len
                    + MAX_PERMITTED_DATA_INCREASE;

                    size += data_len + (data_len as *const u8).align_offset(BPF_ALIGN_OF_U128);
                }
            }
        }

    size += size_of::<u64>() // instruction data len
    + instruction_data.len()
    + size_of::<Pubkey>(); // program id;

    let mut s = Serializer::new(size, MM_INPUT_START);

    s.write::<u64>((utxos.len() as u64).to_le());

    for utxo in utxos {
        match utxo {
            SerializeUtxo::Account(_,mut borrowed_utxo ) => {
                // We write data_len twice because of performance reasons.
                // when we update the length of data inside the program, 
                // we can store current_len in one data entry
                // and orginal_len in other and we can compare if utxo's data
                // is allowed to increase before waiting to compare it in
                // `deserialize_parameters` function

                let data_len = (borrowed_utxo.get_data().len() as u64).to_le();

                s.write::<u8>(NON_DUP_MARKER);
                s.write_all(&[0u8, 0, 0]);
                s.write::<u32>(borrowed_utxo.get_vout());
                s.write_all(borrowed_utxo.get_txid());
                s.write::<u64>(data_len);
                s.write::<u64>(data_len);
                let vm_data_addr = s.write_utxo(&mut borrowed_utxo)?;
                let vm_authority_addr = s.write_all(borrowed_utxo.get_authority().as_ref());

                utxos_metadata.push(SerializedUtxoMetadata {
                    original_data_len : borrowed_utxo.get_data().len(),
                    vm_data_addr,
                    vm_authority_addr
                });
            }
            SerializeUtxo::Duplicate(position) => {
                utxos_metadata.push(utxos_metadata.get(position as usize).unwrap().clone());
                s.write::<u8>(position as u8);
                s.write_all(&[0u8, 0, 0, 0, 0, 0, 0]);
            }
        }
    }
    s.write::<u64>((instruction_data.len() as u64).to_le());
    s.write_all(instruction_data);
    s.write_all(&program_id.as_ref());

    let mem = s.finish();
    Ok((mem,utxos_metadata))
}

pub fn deserialize_parameters(
    transaction_context: &TransactionContext,
    instruction_context: &InstructionContext,
    buffer: &[u8],
    utxo_metadata: &[SerializedUtxoMetadata]
) -> Result<(), InstructionError> {
    let utxo_lengths = utxo_metadata.iter().map(|a| a.original_data_len);

    let mut start = size_of::<u64>(); // number of utxos

    for (instruction_account_index, pre_len) in (0..instruction_context
        .get_number_of_instruction_utxos())
        .zip(utxo_lengths.into_iter()) {

        let duplicate =
            instruction_context.is_instruction_utxo_duplicate(instruction_account_index)?;
        start += size_of::<u8>(); // position
        if duplicate.is_some() {
            start += 7; // padding to 64-bit aligned
        } else {
            let mut borrowed_account = instruction_context
                .try_borrow_instruction_utxo(transaction_context, instruction_account_index)?;
            start += 3 // padding
                + size_of::<u32>()  // vout
                + size_of::<[u8;32]>() // txid of btc txn
                + size_of::<u64>(); // original utxo data len

            let post_len = LittleEndian::read_u64(
                buffer
                    .get(start..)
                    .ok_or(InstructionError::InvalidArgument)?,
            ) as usize;
                
            start += size_of::<u64>(); // updated data length

            if post_len.saturating_sub(pre_len) > MAX_PERMITTED_DATA_INCREASE
                || post_len > MAX_PERMITTED_DATA_LENGTH as usize
            {
                return Err(InstructionError::InvalidRealloc);
            }

            let alignment_offset = (pre_len as *const u8).align_offset(BPF_ALIGN_OF_U128);

            let data = buffer
                    .get(start..start + post_len)
                    .ok_or(InstructionError::InvalidArgument)?;
                match borrowed_account
                    .can_data_be_resized(post_len)
                {
                    Ok(()) => borrowed_account.set_data_from_slice(data)?,
                    Err(err) if borrowed_account.get_data() != data => return Err(err),
                    _ => {}
                }
            start += pre_len; // data
            start += MAX_PERMITTED_DATA_INCREASE;
            start += alignment_offset;
            let authority = buffer
            .get(start..start + size_of::<Pubkey>())
            .ok_or(InstructionError::InvalidArgument)?;

            if borrowed_account.get_authority().to_bytes() != authority {
                // Change the owner at the end so that we are allowed to change the lamports and data before
                borrowed_account.set_authority(authority)?;
            }
        }
    }
    Ok(())
}

mod tests {
    use std::{collections::HashMap, fs::File, io::Read};

    use sha256::digest;

    use crate::processor::{InstructionUtxo, Message, MessageProcessor, TransactionUtxos, UtxoSharedData};

    use super::*;
    
    #[test]
    fn test_serialize_and_back() {
        // make structs:
        use core_types::types::Instruction;
        let instruction_a: Instruction = Instruction {
            program_id: Pubkey([0u8;32]),
            utxos: vec![0,1,2],
            data: vec![1,2,3]
        };
        let message : Message = Message { signers: vec![], instructions: vec![instruction_a] };

        let utxo_a = UtxoSharedData::create(vec![1,1,1,1], Pubkey([0;32]), [2;32], 0);
        let utxo_b = UtxoSharedData::create(vec![2,2,2,2,2], Pubkey([0;32]), [2;32], 1);
           
        let utxos = TransactionUtxos::from(vec![utxo_a,utxo_b]);
        let mut transaction_context : TransactionContext = TransactionContext::new(utxos, 4, 20);

        let mut file = File::open("./compiled-ebpf/simple.so").expect("can't read the elf file");

        let mut elf = Vec::new();
        file.read_to_end(&mut elf).unwrap();

        let mut programs: HashMap<String,Vec<u8>> = HashMap::new();
        programs.insert(digest(digest(Pubkey([0u8;32]).as_ref())), elf);

        let instruction_utxo_a = InstructionUtxo {
            index_in_transaction: 0,
            index_in_caller : 0,
            index_in_callee :0
        };

        let instruction_utxo_b = InstructionUtxo {
            index_in_transaction: 1,
            index_in_caller : 1,
            index_in_callee :1
        };

        let instruction_utxo_c = InstructionUtxo {
            index_in_transaction: 2,
            index_in_caller : 2,
            index_in_callee : 2
        };

        let instruction_context =   InstructionContext {
            nesting_level : 0,
            instruction_data : vec![1,2,3],
            instruction_utxos: vec![instruction_utxo_a, instruction_utxo_b, instruction_utxo_c],
            program_id: Pubkey([0;32]),
        };

        let (parameter_bytes, serialised_accounts) =serialize_parameters(&transaction_context, &instruction_context).expect("Can't serealise");
        // let _ = MessageProcessor::process_message(message, &mut transaction_context, programs).expect("Failed message processing");
    }
}