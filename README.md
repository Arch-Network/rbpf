<<<<<<< Updated upstream
# Rbpf Runtime Environment Testing
=======
# Rbpf Runtiem Environment Testing

## Serialisation Format

## Deserialisation Format

## VM - input/output

# Arch

input to handler function

fn handler(
    program_id: &Pubkey,
    utxos: &[UtxoInfo],
    instruction_data: &[u8],
) -> Result<Transaction>

pub struct UtxoInfo {
    pub txid: String,
    pub vout: u32,
    pub authority: RefCell<Pubkey>,
    pub data: RefCell<Vec<u8>>,
}

Return type of program:
pub struct Transaction {
    pub version: u32,
    pub input: Vec<TxIn>,
    pub output: Vec<TxOut>,
    pub lock_time: u32,
}

pub struct TxIn {
    pub txid: String,
    pub vout: u32,
    pub script_sig: Vec<u8>,
    pub sequence: u32,
    pub witness: Vec<Vec<u8>>,
}

pub struct TxOut {
    pub amount: u64,
    pub script_pubkey: Vec<u8>,
}

```
but output from the journal is deserialised into:

let serialized_res: Vec<u8> = receipt.journal.decode()?;

        #[allow(clippy::type_complexity)]
        let (new_utxo_authorities, new_utxo_data, _unsigned_tx): (
            HashMap<String, Vec<u8>>, (txid of utxo, pubkey of authority)
            HashMap<String, Vec<u8>>, (txid of utxo, data to be put inside the )
            UnsignedTransaction,
        ) = borsh::from_slice(&serialized_res)?;
```

### Input 
** 
Input is 3 things: 
1. serialized Instruction struct
2. authority Hashmap
3. data hashmap
**
            let serialized_instruction: Vec<u8> = env::read();
            let instruction: Instruction = borsh::from_slice(&serialized_instruction).unwrap();
            let program_id: Pubkey = instruction.program_id;
            let authorities: HashMap<String, Vec<u8>> = env::read();
            let data: HashMap<String, Vec<u8>> = env::read();

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
                                .expect("this utxo does not exist")
                                .to_vec(),
                        )),
                        data: RefCell::new(
                            data.get(&utxo.id())
                                .expect("this utxo does not exist")
                                .to_vec(),
                        ),
                    }
                })
                .collect::<Vec<UtxoInfo>>();

### Main code inside the program
***

***
 match $process_instruction(&program_id, &utxos, &instruction_data) {
                Ok(tx_hex) => {
                    let mut new_authorities: HashMap<String, Vec<u8>> = HashMap::new();
                    let mut new_data: HashMap<String, Vec<u8>> = HashMap::new();
                    utxos.iter().for_each(|utxo| {
                        new_authorities.insert(utxo.id(), utxo.authority.clone().into_inner().0);
                        new_data.insert(utxo.id(), utxo.data.clone().into_inner());
                    });
                    env::commit(&borsh::to_vec(&(new_authorities, new_data, tx_hex)).unwrap())
                }
                Err(e) => panic!("err: {:?}", e),
            }
>>>>>>> Stashed changes
