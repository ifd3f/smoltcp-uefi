# smoltcp-uefi

This crate contains utilities for using the [smoltcp](https://crates.io/crates/smoltcp) crate inside an EFI environment.

**WARNING:** This crate is highly experimental right now! It's more of a proof-of-concept than something that's production ready! Suggestions and pull requests are welcome.

## Features

- `SnpDevice`, a `smoltcp::phy::Device` running on UEFI's Simple Network Protocol.
- Type conversion utilities
- Utilities for getting monotonic `smoltcp::time::Instant`s inside UEFI

## Example code

There is currently [one example that implements ipv4 ping](./examples/ping.rs).

It runs in UEFI, so you probably shouldn't `cargo run` it on a normal system. Instead, I've provided a helper script in [scripts/test_on_qemu.sh](./scripts/test_on_qemu.sh) for building and running it. See the comment inside that script for information on how to run it.
