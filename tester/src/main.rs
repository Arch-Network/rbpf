
use std::{collections::HashMap, ptr::slice_from_raw_parts, slice::from_raw_parts_mut};

use test::{test_v2};
mod vm;
pub mod test;
mod ebpffile;
pub mod config;
mod cpi;
mod processor;
fn main() {
    // test_everything();
    test_v2()
}