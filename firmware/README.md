# Firmware

## Components

The firmware consists of three parts.
The brain interface firmware can be found in the [`brain-interface`](brain-interface) directory.
The firmware for the dongle is in the [`dongle`](dongle) directory.
Some shared components are in the [`data-channel`](data-channel) directory.

## Prerequisites

To compile and flash the firmware you will need the Rust compiler.
You can install it using the instructions on [the rustup website](https://rustup.rs/).
This will install the `rustup` tool which is used to install different versions of the Rust toolchain.

To actually flash and run the firmware you will need the `probe-rs` tool.
You can install it by using the command `cargo install probe-rs --features cli`.

For debug we are using the `probe-run` tool.
Install it with `cargo install probe-run`.

## Compiling

After all tools are installed the firmware can be compiled by executing `cargo build --release` in the firmware directory.
This will build both the brain interface and the dongle firmware.
The firmware must be built in release mode or it will have performance issues.

## Debugging

You can also run the firmware with an attached debugger.
To do this use `DEFMT_LOG=trace cargo run --release -p brain-interface`.
The `DEFMT_LOG` environment variable selects the log level that the firmware prints to the debug terminal.
It defaults to `error`, so to see all messages we use the value `trace` instead.
All possible levels in increasing order are `trace`, `debug`, `info`, `warn` and `error`.

The `-p` flag selects the firmware to debug, build or flash.
For the dongle firmware use `-p dongle` instead of `-p brain-interface`.

To flash the firmware for production, use the following command: `cargo flash --release --chip nRF52840_xxAA -p brain-interface`.

## Documentation

To build the documentation for the project, run `cargo doc`.
You can then find the generated documentation files in the `target/thumbv7em-none-eabihf/doc` directory.
The can be viewed in any web browser.
