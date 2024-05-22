use std::sync::Arc;
use crate::consts::PUBKEY_BYTES;

pub struct AccountSharedData {
    /// lamports in the account
    lamports: u64,
    /// data held in this account
    data: Arc<Vec<u8>>,
    /// the program that owns this account. If executable, the program that loads this account.
    owner: Pubkey,
    /// this account's data contains a loaded program (and is now read-only)
    executable: bool,
    /// the epoch at which this account will next owe rent
    /// TODO: We need some concept for prevention against DOS attack on new accounts that can fill
    /// our account database
    rent_epoch: u64,
}

#[derive(Debug)]
pub struct Pubkey(pub(crate) [u8; 32]);
impl Pubkey {
    fn from_array(arr: [u8; 32]) -> Self {
        Pubkey(arr)
    }
    fn into_array(&self) -> [u8; 32] {
        self.0
    }
}

pub fn new_rand() -> Pubkey {
    Pubkey::from(rand::random::<[u8; PUBKEY_BYTES]>())
}

#[derive(Debug)]
pub struct UtxoInfo {
    pub txid: [u8; 32],
    pub vout: u32,
    pub value: u64,
}

impl UtxoInfo {
    fn from_data(txid: [u8; 32], vout: u32, value: u64) -> Self {
        UtxoInfo { txid, vout, value }
    }
    fn random() -> Self {
        UtxoInfo {
            txid: [0; 32],
            vout: 20,
            value: 40,
        }
    }
}

mod Test {
    use super::*;

    #[test]
    fn test_pubkey_gen() {
        assert!(new_rand().into_array().len() == 32);
    }
}