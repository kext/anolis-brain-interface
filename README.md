# Anolis Brain Interface

This repository contains the hardware and software files for the Anolis Brain Interface.

## PCB

The source and production files for the flexible PCB are in the `pcb` directory.
See the [Readme there](pcb/README.md) for instructions to setup your work environment.

## Firmware

The firmware for the brain interface is in the `main-firmware` directory.

To compile and flash it you will need the Rust compiler.
You can install it using the instructions on [the rustup website](https://rustup.rs/).
This will install the `rustup` tool which is used to install different versions of the Rust toolchain.

To actually flash and run the firmware you will need the `probe-rs` tool.
You can install it by using the command `cargo install probe-rs --features cli`.

For debug we are using the `probe-run` tool.
Install it with `cargo install probe-run`.

After all tools are installed the firmware can be compiled by executing `cargo build --release` in the firmware directory.
The firmware must be built in release mode or it will have performance issues.

You can also run the firmware with an attached debugger.
To do this use `DEFMT_LOG=trace cargo run --release`.
The `DEFMT_LOG` environment variable selects the log level that the firmware prints to the debug terminal.
It defaults to `error`, so to see all messages we use the value `trace` instead.
All possible levels in increasing order are `trace`, `debug`, `info`, `warn` and `error`.
