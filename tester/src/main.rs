
use std::{collections::HashMap, ptr::slice_from_raw_parts, slice::from_raw_parts_mut};
mod vm;
pub mod test;
mod ebpffile;
use test::test_everything;
pub mod config;


fn main() {
    // println!("Hey");
test_everything()
}