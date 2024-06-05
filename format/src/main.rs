use std::{fs::File, io::Read};
 use solana_rbpf::elf_parser::Elf64;

fn main() {
    let mut file = File::open("../ebpf.so").expect("unable to open file");
    let mut elf = Vec::new();
    file.read_to_end(&mut elf).unwrap();
    let a =Elf64::parse(&elf).expect("unable to parse elf file");

    println!("{:?}",a);
}  
