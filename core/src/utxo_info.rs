use core::fmt;
use std::{cell::{Ref, RefCell, RefMut}, rc::Rc, slice::from_raw_parts_mut};

use crate::{entrypoint::MAX_PERMITTED_DATA_INCREASE, types::Pubkey, UtxoIdentity};

use crate::{debug_utxo_data::debug_account_data, program_error::ProgramError};

// Account information
#[derive(Clone)]
#[repr(C)]
pub struct UtxoInfo<'a> {
    /// The data held in this utxo.  Modifiable by programs.
    pub data: Rc<std::cell::RefCell<&'a mut [u8]>>,
    /// Program that owns this utxo
    pub authority: &'a Pubkey,
    /// txid of btc transaction
    pub txid: &'a [u8;32],
    /// vout in btc txn
    pub vout: u32
}

impl<'a> UtxoIdentity for UtxoInfo<'a> {
    fn utxo_id(&self) -> crate::UtxoId {
        <Self as UtxoIdentity>::processor(&self.txid, self.vout)
    }
}

impl<'a> fmt::Debug for UtxoInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_struct("UtxoInfo");

            f.field("txid", &self.txid)
            .field("vout", &self.vout)
            .field("authority", &self.authority)
            .field("data.len", &self.data_len());
        debug_account_data(&self.data.borrow(), &mut f);

        f.finish_non_exhaustive()
    }
}

impl<'a> UtxoInfo<'a> {

    pub fn new(
        data: &'a mut [u8],
        authority : &'a Pubkey,
        txid: &'a [u8;32],
        vout: u32
    ) -> Self {
        Self {
            data: Rc::new(RefCell::new(data)),
            authority,
            txid,
            vout
        }
    }
    
    pub fn data_len(&self) -> usize {
        self.data.borrow().len()
    }

    pub fn try_borrow_data(&self) -> Result<Ref<&mut [u8]>, ProgramError> {
        self.data
            .try_borrow()
            .map_err(|_| ProgramError::AccountBorrowFailed)
    }

    pub fn data_is_empty(&self) -> bool {
        self.data.borrow().is_empty()
    }

    pub fn try_borrow_mut_data(&self) -> Result<RefMut<&'a mut [u8]>, ProgramError> {
        self.data
            .try_borrow_mut()
            .map_err(|_| ProgramError::AccountBorrowFailed)
    }

    /// Return the utxo's original data length when it was serialized for the
    /// current program invocation.
    ///
    /// # Safety
    ///
    /// This method assumes that the original data length was serialized as a u64
    /// integer in the 8 bytes immediately succeeding the serialized txid.
    pub unsafe fn original_data_len(&self) -> usize {
        let txid_ptr = self.txid as *const _ as *const u8;
        let original_data_len_ptr = txid_ptr.offset(32) as *const u64;
        *original_data_len_ptr as usize
    }

    /// Realloc the account's data and optionally zero-initialize the new
    /// memory.
    ///
    /// Note:  Account data can be increased within a single call by up to
    /// `solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE` bytes.
    ///
    /// Note: Memory used to grow is already zero-initialized upon program
    /// entrypoint and re-zeroing it wastes compute units.  If within the same
    /// call a program reallocs from larger to smaller and back to larger again
    /// the new space could contain stale data.  Pass `true` for `zero_init` in
    /// this case, otherwise compute units will be wasted re-zero-initializing.
    ///
    /// # Safety
    ///
    /// This method makes assumptions about the layout and location of memory
    /// referenced by `AccountInfo` fields. It should only be called for
    /// instances of `AccountInfo` that were created by the runtime and received
    /// in the `process_instruction` entrypoint of a program.
    pub fn realloc(&self, new_len: usize, zero_init: bool) -> Result<(), ProgramError> {
        let mut data = self.try_borrow_mut_data()?;
        let old_len = data.len();

        // Return early if length hasn't changed
        if new_len == old_len {
            return Ok(());
        }

        // Return early if the length increase from the original serialized data
        // length is too large and would result in an out of bounds allocation.
        let original_data_len = unsafe { self.original_data_len() };
        if new_len.saturating_sub(original_data_len) > MAX_PERMITTED_DATA_INCREASE {
            return Err(ProgramError::InvalidRealloc);
        }

        // realloc
        unsafe {
            let data_ptr = data.as_mut_ptr();

            // First set new length in the serialized data
            *(data_ptr.offset(-8) as *mut u64) = new_len as u64;

            // Then recreate the local slice with the new length
            *data = from_raw_parts_mut(data_ptr, new_len)
        }

        if zero_init {
            let len_increase = new_len.saturating_sub(old_len);
            if len_increase > 0 {
                let mut data = &mut data[old_len..];
                data.fill(0);
            }
        }

        Ok(())
    }

    #[rustversion::attr(since(1.72), allow(invalid_reference_casting))]
    pub fn assign(&self, new_authority: &Pubkey) {
        // Set the non-mut owner field
        unsafe {
            std::ptr::write_volatile(
                self.authority as *const Pubkey as *mut [u8; 32],
                new_authority.to_bytes(),
            );
        }
    }
}