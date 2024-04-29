#!/bin/bash -ex

# Requires Latest release of Solana's custom LLVM
# https://github.com/solana-labs/platform-tools/releases

TOOLCHAIN="/Users/deepanshuhooda/Downloads/Platform"
RC_COMMON="$TOOLCHAIN/rust/bin/rustc --target sbf-solana-solana --crate-type lib -C panic=abort -C opt-level=2"
RC="$RC_COMMON -C target_cpu=sbfv2"
RC_V1="$RC_COMMON -C target_cpu=generic"
LD_COMMON="$TOOLCHAIN/llvm/bin/ld.lld -z notext -shared --Bdynamic -entry entrypoint --script elf.ld"
LD="$LD_COMMON --section-start=.text=0x100000000"
LD_V1=$LD_COMMON

$RC -o relative_call.o relative_call.rs
$LD -o relative_call.so relative_call.o