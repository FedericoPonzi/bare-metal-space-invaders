#!/usr/bin/env bash

KERNEL_ELF=target/aarch64-unknown-none-softfloat/release/bare-metal-spaceinvaders

## build the kernel
RUSTFLAGS="-C link-arg=-T$(pwd)/linker/rpi_3b+.ld -C target-cpu=cortex-a53" cargo rustc \
        --manifest-path kernel/Cargo.toml \
        --target=aarch64-unknown-none-softfloat \
        --release

## strip and get the kernel8 image
rust-objcopy \
    --strip-all \
    -O binary \
    $KERNEL_ELF \
    kernel8.img \

