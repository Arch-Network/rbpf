use std::io::Read;

pub mod entrypoint;
pub mod types;
// pub mod dentrypoint;
mod debug_utxo_data;
mod log;
mod processor;
mod program;
mod program_error;
mod stable_vec;
mod syscalls;
mod test;
pub mod utxo_info;

#[derive(Debug, PartialEq, Eq, Clone)]
#[repr(C)]
pub struct UtxoId(pub [u8; 36]);
pub trait UtxoIdentity {
    fn utxo_id(&self) -> UtxoId;

    fn processor(txn_id: &[u8; 32], vout: u32) -> UtxoId {
        let mut result = [0_u8; 36];
        result[..txn_id.len()].copy_from_slice(txn_id);
        result[txn_id.len()..].copy_from_slice(&vout.to_le_bytes());
        result.into()
    }
}

impl From<[u8; 36]> for UtxoId {
    fn from(value: [u8; 36]) -> Self {
        UtxoId(value)
    }
}

mod Test {
    use super::*;

    struct TestStruct {
        pub txid: [u8; 32],
        pub vout: u32,
    }

    impl UtxoIdentity for TestStruct {
        fn utxo_id(&self) -> UtxoId {
            <TestStruct as UtxoIdentity>::processor(&self.txid, self.vout)
        }
    }

    #[test]
    fn test_utxo_identity_trait() {
        let tester = TestStruct {
            txid: [1; 32],
            vout: 15,
        };
        let mut result = [0u8; 36];

        // Copy elements of array1
        result[..tester.txid.len()].copy_from_slice(&tester.txid);

        // Copy elements of array2
        result[tester.txid.len()..].copy_from_slice(&tester.vout.to_le_bytes());

        assert_eq!(tester.utxo_id(), result.into())
    }
}
