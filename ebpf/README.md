Name of elf file and their roles
1. authority-and-data-modifier.so
```pub fn process_instruction(key: &Pubkey, utxos: &[UtxoInfo], ins: &[u8]) -> Result<(), String> {
    let first_utxo = &utxos[0];
    let second_utxo = &utxos[1];
    first_utxo.data.borrow_mut()[0] = 65;
    first_utxo.data.borrow_mut()[1] = 15;



    first_utxo.assign(&[22;32].into());

    return Ok(());
}
```

2. cpi.so

```
pub fn process_instruction(key: &Pubkey, utxos: &[UtxoInfo], ins: &[u8]) -> Result<(), String> {
    let first_utxo = &utxos[0].utxo_id();
    let second_utxo = &utxos[1].utxo_id();

    let mut program_id = [0u8;32];
    program_id.clone_from_slice(ins);

    let utxo2 = &utxos[2];
    utxo2.assign(&program_id.into());

    invoke(&ProgramInstruction::from(program_id.into(),vec![first_utxo.clone(), second_utxo.clone()],vec![0]),&[utxos[0].clone(), utxos[1].clone()]).expect("failed at cpi");

    return Ok(());
}
```