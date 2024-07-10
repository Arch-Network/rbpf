
use std::{collections::HashMap, ptr::slice_from_raw_parts, slice::from_raw_parts_mut};

mod vm;
pub mod test;
mod ebpffile;
pub mod config;
mod cpi;
mod processor;
mod serialization;
mod errors;
mod program_error;
mod syscall_error;
fn main() {

}