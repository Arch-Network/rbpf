use std::str::FromStr;

use anyhow::{anyhow, Result};
use bitcoin::Address;
use borsh::{BorshDeserialize, BorshSerialize};
use core_types::{
    entrypoint,
    types::{Pubkey, Transaction, TxIn, TxOut, UtxoInfo},
};

pub fn handler(
    program_id: &Pubkey,
    utxos: &[UtxoInfo],
    instruction_data: &[u8],
) -> Result<Transaction> {
    if instruction_data[0] == 0 {
        init_bridge(program_id, utxos, &instruction_data[1..])
    } else if instruction_data[0] == 1 {
        bridge_pegin(program_id, utxos, &instruction_data[1..])
    } else if instruction_data[0] == 2 {
        bridge_pegout(program_id, utxos, &instruction_data[1..])
    } else {
        Err(anyhow!("invalid instruction!"))
    }
}

fn init_bridge(
    program_id: &Pubkey,
    utxos: &[UtxoInfo],
    instruction_data: &[u8],
) -> Result<Transaction> {
    let params: InitBridgeParams = borsh::from_slice(&instruction_data)?;

    let bridge_state = BridgeState { vaults: Vec::new() };

    *utxos[0].data.borrow_mut() = borsh::to_vec(&bridge_state)?;

    Ok(Transaction {
        version: 2,
        input: vec![TxIn {
            txid: params.fee_txid,
            vout: params.fee_vout,
            sequence: 0,
            script_sig: program_id.to_array().to_vec(),
            witness: vec![],
        }],
        output: vec![],
        lock_time: 0,
    })
}

#[derive(Clone, Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct InitBridgeParams {
    pub fee_txid: String,
    pub fee_vout: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct Vault {
    pub token: String,
    pub amount: u64,
    pub receiver_address: String,
    pub sent_at: u64,
    pub txid: String,
    pub vout: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct BridgeState {
    pub vaults: Vec<Vault>,
}

fn bridge_pegin(
    program_id: &Pubkey,
    utxos: &[UtxoInfo],
    instruction_data: &[u8],
) -> Result<Transaction> {
    let mut bridge_state: BridgeState = borsh::from_slice(&utxos[0].data.borrow())?;

    let params: BridgePegInParams = borsh::from_slice(instruction_data)?;

    bridge_state.vaults.push(Vault {
        txid: params.txid,
        vout: params.vout,
        token: params.token,
        amount: params.amount,
        receiver_address: params.receiver_address,
        sent_at: 0,
    });

    *utxos[0].data.borrow_mut() = borsh::to_vec(&bridge_state)?;

    Ok(Transaction {
        version: 2,
        input: vec![TxIn {
            txid: params.fee_txid,
            vout: params.fee_vout,
            sequence: 0,
                script_sig: program_id.to_array().to_vec(),
                witness: vec![],
        }],
        output: vec![],
        lock_time: 0,
    })
}

#[derive(Clone, Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct BridgePegInParams {
    pub token: String,
    pub amount: u64,
    pub receiver_address: String,
    pub txid: String,
    pub vout: u32,
    pub fee_txid: String,
    pub fee_vout: u32,
}

fn bridge_pegout(
    program_id: &Pubkey,
    utxos: &[UtxoInfo],
    instruction_data: &[u8],
) -> Result<Transaction> {
    let _bridge_state: BridgeState = borsh::from_slice(&utxos[0].data.borrow())?;

    let params: BridgePegOutParams = borsh::from_slice(instruction_data)?;

    Ok(Transaction {
        version: 2,
        input: vec![
            TxIn {
                txid: params.txid,
                vout: params.vout,
                sequence: 0,
                script_sig: program_id.to_array().to_vec(),
                witness: vec![],
            },
            TxIn {
                txid: params.fee_txid,
                vout: params.fee_vout,
                sequence: 0,
                script_sig: program_id.to_array().to_vec(),
                witness: vec![],
            },
        ],
        output: vec![TxOut {
            amount: params.value,
            script_pubkey: Address::from_str(&params.receiver_address)
                .expect("bitcoin address must be valid")
                .require_network(bitcoin::Network::Bitcoin)
                .expect("bitcoin address must be from bitcoin network")
                .script_pubkey()
                .as_bytes()
                .to_vec(),
        }],
        lock_time: 0,
    })
}

#[derive(Clone, Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct BridgePegOutParams {
    pub txid: String,
    pub vout: u32,
    pub value: u64,
    pub receiver_address: String,
    pub fee_txid: String,
    pub fee_vout: u32,
}
