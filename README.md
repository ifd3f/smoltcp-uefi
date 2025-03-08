# smoltcp-uefi

This crate contains utilities for using the [smoltcp](https://crates.io/crates/smoltcp) crate inside an EFI environment.

## Example code

There is currently [one example that implements ipv4 ping](./examples/ping.rs).

It runs in UEFI, so you probably shouldn't `cargo run` it on a normal system. Instead, I've provided a helper script in [scripts/test_on_qemu.sh](./scripts/test_on_qemu.sh) for building and running it. See the comment inside that script for information on how to run it.
