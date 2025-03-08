#!/usr/bin/env bash

# Helper script to test the ping example on a QEMU VM.
# Open up Wireshark, run this script, and watch the packets go by.
# 
# Prerequisites:
# - You should copy OVMF_CODE and OVMF_VARS.fd into the working directory
#   before running this.
# - This assumes you already have a bridge set up named virbr0.

set -euxo pipefail

# set up an esp as a VVFAT directory
cargo build --example ping --target x86_64-unknown-uefi --release
rm -rf esp
mkdir -p esp/efi/boot
cp -r target/x86_64-unknown-uefi/release/examples/ping.efi esp/efi/boot/bootx64.efi

# launch!
sudo qemu-system-x86_64 \
    -D ./qemu.log \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.fd \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_VARS.fd \
    -nic bridge,mac=52:54:00:69:69:69,br=virbr0 \
    -drive format=raw,file=fat:rw:esp
    # -drive if=virtio,file=ipxe.iso # test if networking works like, at all