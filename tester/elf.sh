#!/bin/bash -ex

# Requires Latest release of Solana's custom LLVM
# https://github.com/solana-labs/platform-tools/releases

TOOLCHAIN="/Users/deepanshuhooda/Downloads/platform-tools-osx-aarch64"
RC_COMMON="$TOOLCHAIN/rust/bin/rustc --target sbf-solana-solana --crate-type lib -C panic=abort -C opt-level=2"
RC="$RC_COMMON -C target_cpu=sbfv2"
RC_V1="$RC_COMMON -C target_cpu=generic"
LD_COMMON="$TOOLCHAIN/llvm/bin/ld.lld -z notext -shared --Bdynamic -entry entrypoint --script elf.ld"
LD="$LD_COMMON --section-start=.text=0x100000000"
LD_V1=$LD_COMMON

$RC -o elffile.o src/ebpffile.rs
$LD -o elffile.so elffile.o

rm *.o